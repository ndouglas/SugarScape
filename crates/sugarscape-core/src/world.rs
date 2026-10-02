//! The world: lattice, agents, and the tick loop.

use std::collections::BTreeMap;

use rand::seq::SliceRandom;
use rand::Rng;

use crate::agent::{Agent, AgentId, DiseaseId, Tribe};
use crate::bits::Bits;
use crate::config::{Config, FieldError, FounderAges, Placement, Who, MAX_GOODS};
use crate::geometry::{Pos, Torus};
use crate::landscape::{self, Site};
use crate::minds::spatial_hoarding::state::{FounderTraits, SpatialState};
use crate::rng::{self, SimRng};
use crate::rules;
use crate::stats::{Snapshot, Stats};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeathCause {
    Starvation,
    OldAge,
    Combat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Death {
    pub id: AgentId,
    pub tribe: Tribe,
    pub cause: DeathCause,
}

/// One exchange under rule T: `buyer` received `amount` units of good
/// `goods.0` and paid `amount × price` units of good `goods.1` to `seller`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Trade {
    pub buyer: AgentId,
    pub seller: AgentId,
    pub goods: (usize, usize),
    pub price: f64,
    pub amount: f64,
}

/// One kill under rule C: `attacker` took `victim`'s site and `loot` of its
/// sugar.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Kill {
    pub attacker: AgentId,
    pub victim: AgentId,
    pub loot: f64,
}

/// One infection: `infector` gave `disease` to `infected` (`None` for an
/// outbreak).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Infection {
    pub infector: Option<AgentId>,
    pub infected: AgentId,
    pub disease: DiseaseId,
}

pub type LoanId = u64;

/// Most positions a trail keeps; the oldest are dropped first.
pub const TRAIL_LEN: usize = 500;

/// A loan of good `good` under rule L: `due` of it owed at `due_tick`, written for
/// `duration` ticks at `rate` percent per tick (the terms travel with it).
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
pub struct Loan {
    pub id: LoanId,
    pub lender: AgentId,
    pub borrower: AgentId,
    pub good: usize,
    pub principal: f64,
    pub due: f64,
    pub due_tick: u64,
    pub duration: u32,
    pub rate: f64,
}

/// What happened during the current (or last completed) tick.
#[derive(Clone, Debug, Default)]
pub struct TickEvents {
    pub births: u32,
    pub deaths: Vec<Death>,
    pub trades: Vec<Trade>,
    pub kills: Vec<Kill>,
    pub loans_made: u32,
    pub amount_lent: f64,
    pub defaults: u32,
    pub infections: Vec<Infection>,
    /// Minds 3: truffle spots harvested this tick.
    pub truffles_found: u32,
    /// Minds 3: of `truffles_found`, how many were harvested by a rememberer.
    pub truffles_by_rememberers: u32,
    /// Minds 3: choices of where to go made this tick by rememberers.
    pub moves: u32,
    /// Minds 3: of `moves`, those whose target was a remembered site out of
    /// sight.
    pub remembered_moves: u32,
    /// Minds 3: Σ |believed − true| welfare over the `remembered_moves`,
    /// measured when the target is chosen (not on arrival). Remembered
    /// values carry no pollution discount, so under pollution the error
    /// includes pollution the agent couldn't see.
    pub belief_error_sum: f64,
    /// Minds 3: of `remembered_moves`, those whose target was truly worth
    /// less than believed when chosen.
    pub stale_choices: u32,
    /// Minds 4: GOAP plans made this tick (found, with at least one step;
    /// the fallback and an empty plan aren't counted).
    pub plans: u32,
    /// Minds 4: Σ steps over this tick's `plans`.
    pub plan_steps_sum: u32,
    /// Minds 4: of `plans`, those with any target from the agent's
    /// remembered entries out of sight (the usage check Minds 3 taught).
    pub plans_with_remembered: u32,
    /// Minds 4: of `plans`, those made by rememberers (the denominator of
    /// the rememberers' usage share; only a rememberer's plan can hold a
    /// remembered site).
    pub plans_by_rememberers: u32,
    /// Minds 4: GOAP agents that took the rate choice this tick because the
    /// sugar they know of (their slots, all harvested) falls short of G.
    pub fallback_short: u32,
    /// Minds 4: GOAP agents that took the rate choice this tick because the
    /// search passed `PLAN_LIMIT` expansions.
    pub fallback_limit: u32,
    /// Minds 4: MVT agents that set `leaving` this tick (their local value
    /// fell below ρ).
    pub leaves: u32,
    /// Minds 5: sugar buried into caches this tick.
    pub buried: f64,
    /// Minds 5: sugar dug out of caches this tick.
    pub dug: f64,
    /// Minds 6: sugar lost to `caching.bury_cost` this tick (counted as
    /// eaten).
    pub bury_cost: f64,
    /// Minds 5: sugar left in the caches of agents removed this tick (it
    /// leaves the world with them).
    pub cache_lost: f64,
    /// Minds 9 per-kind flows; absent on ordinary worlds.
    pub spatial_stores: Option<crate::minds::spatial_hoarding::stores::StoreEvents>,
    /// Minds 5: Σ over this tick's `digs` of the dug cache's age (ticks since
    /// its first unit was buried).
    pub dig_ages_sum: u64,
    /// Minds 5: digs this tick (each taking a positive amount).
    pub digs: u32,
    /// Minds 6: sugar pilfered from caches this tick (under either loot
    /// rule; owners finding their own caches under `owner_memory: off` are
    /// digs, not pilfers).
    pub pilfered: f64,
    /// Minds 6: pilfers this tick (takes, each of a positive amount).
    pub pilfers: u32,
    /// Minds 6: distinct caches, (owner, site), that existed at the tick's
    /// start and were pilfered this tick, in any amount: each counted once,
    /// however many thieves took from it and whether a take emptied it. The
    /// spec's pilferage rate is `caches_pilfered / pilfer_candidates`. A
    /// cache begun this tick (its `cache_since` is the current tick) wasn't
    /// there at the start and isn't counted.
    pub caches_pilfered: u32,
    /// The caches counted in `caches_pilfered` this tick (for the once
    /// only); empty and unallocated unless something was pilfered.
    pub(crate) pilfered_caches: std::collections::BTreeSet<(
        AgentId,
        u32,
        crate::minds::spatial_hoarding::state::StoreKind,
    )>,
    /// Minds 6: caches in the world at the tick's start (after the
    /// schedule), counted under theft or watching (`pilfering_on()`): Σ over agents of
    /// their caches. Each is a foreign cache to every agent but its owner,
    /// so `pilfers / pilfer_candidates` is the per-cache pilfer rate.
    pub pilfer_candidates: u32,
    /// Minds 6: under `owner_memory: off`, owners who found (and dug) their
    /// own cache this tick. Counted in `digs` and `dug` too.
    pub owner_finds: u32,
    /// Minds 6: find draws made this tick on other agents' caches: each is
    /// a non-owner's visit to a cache that could find it (an arrival that
    /// didn't dig its own cache there). Draws on an agent's own cache under
    /// `owner_memory: off` aren't counted. Observation only (the survey's
    /// visits per cache); it draws nothing itself.
    pub pilfer_draws: u32,
    /// Minds 6: of `pilfered`, the sugar eaten under `theft.loot: eat`,
    /// counted as it goes into the thief's stomach (`Agent::fed`). A
    /// transfer, not a ledger term: the stomach is a stock, and the
    /// metabolism drawing on it is what's eaten.
    pub loot_eaten: f64,
    /// Minds 6: sugar left in the stomachs (`Agent::fed`) of agents removed
    /// this tick (it leaves the world with them, like `cache_lost`).
    pub fed_lost: f64,
    /// Minds 5, central-place foraging: loads delivered home this tick (a
    /// delivery is a positive burial into the larder by an agent back from
    /// a trip).
    pub deliveries: u32,
    /// Minds 5, central-place foraging: Σ over this tick's `deliveries` of
    /// the load buried (each trip's load size).
    pub delivered: f64,
    /// Minds 8: burials this tick seen by at least one watcher.
    pub burials_seen: u32,
    /// Minds 8: (watcher, burial) pairs this tick.
    pub sightings: u32,
    /// Minds 8: seen-cache entries held, summed over agents, after the
    /// tick-start sweep (entries forgotten later in the tick, or made
    /// during it, aren't reflected). 0 with `watching.on` false.
    pub seen_entries: u32,
    /// Minds 8: takes from a seen cache (raids) this tick. Each is also a
    /// pilfer, counted in `pilfers`.
    pub raids: u32,
    /// Minds 8: the sugar raids took this tick, also counted in `pilfered`.
    pub raided: f64,
    /// Minds 8: arrivals whose remembered caches at the site were all gone
    /// (wasted raids), once per arrival.
    pub raids_wasted: u32,
    /// Minds 8: arrivals on a site where the agent remembered a seen cache
    /// (each forgets its entries there, whether it took or not).
    pub seen_arrivals: u32,
}

#[derive(Clone)]
pub struct World {
    pub config: Config,
    pub torus: Torus,
    /// Completed ticks.
    pub tick: u64,
    pub sites: Vec<Site>,
    /// Minds 3: truffle spots, row-major, one entry per site. `None` is no
    /// spot; `Some(t)` is a spot ripe again at tick `t` (ripe now when
    /// `t <= tick`). Built once from `config.truffles` (placed by hash, not
    /// `World.rng`; walls never get a spot); empty when `truffles.share` is
    /// 0, so `truffle` answers `None` everywhere without a lookup.
    pub truffles: Vec<Option<u64>>,
    /// Chapter V's master list of diseases; a disease's id is its index.
    pub diseases: Vec<Bits>,
    agents: BTreeMap<AgentId, Agent>,
    /// Checked supplied traits, retained for founder statistics after deaths.
    /// None in ordinary worlds; slot order is ascending founder id.
    pub(crate) spatial_cohort: Option<Vec<FounderTraits>>,
    occupancy: Vec<Option<AgentId>>,
    /// Row-major, one entry per site: 0 free, 1 a fence, 2 opaque. Built once
    /// from `config.walls` (walls change only on reset, except the Minds 5
    /// lab's doorways, which `open_wall` opens on the test evening); all zero
    /// when there are none.
    pub(crate) walls: Vec<u8>,
    /// Row-major: each non-wall site's connected component among the
    /// non-wall sites (4-way, on the torus); walls get `u32::MAX`. Built once
    /// with `walls` (and again by `open_wall`); empty when there are none,
    /// and then never consulted.
    pub(crate) regions: Vec<u32>,
    pub(crate) rng: SimRng,
    next_id: AgentId,
    pub(crate) events: TickEvents,
    pub stats: Stats,
    loans: BTreeMap<LoanId, Loan>,
    next_loan_id: LoanId,
    /// The agent whose trail is recorded (observation only: never hashed,
    /// exported or put in configs).
    followed: Option<AgentId>,
    /// Its positions after each tick, oldest first.
    trail: Vec<Pos>,
    /// Minds 6: every cache's fate, one record per burial event
    /// (`minds::caching::fates`). Recorded only when a caller asks for it
    /// (`record_fates`), and then only under theft or watching
    /// (`pilfering_on()`);
    /// otherwise empty and unallocated. Never hashed.
    pub cache_log: Vec<crate::minds::caching::fates::CacheRecord>,
    /// Minds 6: the log reached `fates::LOG_CAP` and froze.
    pub cache_log_full: bool,
    /// Minds 6, a survey probe: when true, a field agent with caches digs
    /// below its whole reserve R instead of R / 2 (no hysteresis band;
    /// `minds::caching::hungry`). Not config: never set by a config, the app
    /// or an edit, never hashed or exported, and false in every world the
    /// survey doesn't set it in. Minds 8b made it a setting too,
    /// `caching.dig_below: reserve`, which runs identically.
    #[doc(hidden)]
    pub probe_dig_at_reserve: bool,
    /// Minds 8, a survey probe: when true, a raid that took something
    /// (`minds::caching::watching::raid`) also harvests the site that tick,
    /// as the ordinary harvest does (under the carrying limit, the rest left
    /// on the site, counted in `gathered`), instead of replacing it. Not
    /// config: never set by a config, the app or an edit, never hashed or
    /// exported, and false in every world the survey doesn't set it in.
    /// Under it a tick can both pilfer and gather (`Harvest::pilfered` and
    /// `gathered` both positive), and Compensate's weight update is skipped
    /// on such ticks, as on every tick that pilfered.
    #[doc(hidden)]
    pub probe_raid_harvests: bool,
    /// Minds 6: when true, the world keeps its fate log (`cache_log`) under
    /// theft. Not config: never set by a config, the app or an edit, never
    /// hashed or exported, and false unless a caller (the survey, a test)
    /// sets it, so the app and sweeps never grow the log. The statistics
    /// come from the tick events, not the log, and don't depend on it.
    #[doc(hidden)]
    pub record_fates: bool,
    /// Minds 6: the log's open records per (owner, site), oldest first.
    pub(crate) cache_open: crate::minds::caching::fates::OpenRecords,
    /// Minds 6: site index → the owners of caches there, for finding
    /// foreign caches on arrival (`minds::caching::theft`). `None` until the
    /// first arrival that needs it under `theft.find > 0`, which builds it
    /// from every agent's caches; then kept by bury, dig, pilfer and
    /// removal, and dropped (back to `None`) by the first of those after
    /// `find` goes to 0. Never hashed; worlds without theft never build it.
    pub(crate) cache_sites: Option<crate::minds::caching::theft::CacheSites>,
}

impl World {
    pub fn new(config: Config, seed: u64) -> Result<Self, Vec<FieldError>> {
        Self::with_capacities(config, seed, None)
    }

    /// A spatial episode with checked per-slot traits and strategy flags.
    /// Slots follow ascending founder ids and are applied before tick zero.
    pub fn new_with_spatial_cohort(
        config: Config,
        seed: u64,
        cohort: &[FounderTraits],
    ) -> Result<Self, Vec<FieldError>> {
        Self::initialize(config, seed, &[], Some(cohort))
    }

    /// Like `new`, with good 0's capacities supplied (a painted map from a
    /// pre-N-goods share link).
    pub fn with_capacities(
        config: Config,
        seed: u64,
        capacities: Option<&[f64]>,
    ) -> Result<Self, Vec<FieldError>> {
        Self::with_landscapes(config, seed, &[capacities.map(<[f64]>::to_vec)])
    }

    /// Like `new`, with each good's row-major capacities supplied, or
    /// generated from its map where the entry is `None` or missing.
    pub fn with_landscapes(
        config: Config,
        seed: u64,
        landscapes: &[Option<Vec<f64>>],
    ) -> Result<Self, Vec<FieldError>> {
        Self::initialize(config, seed, landscapes, None)
    }

    fn initialize(
        config: Config,
        seed: u64,
        landscapes: &[Option<Vec<f64>>],
        cohort: Option<&[FounderTraits]>,
    ) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        if let Some(cohort) = cohort {
            let mut errors = Vec::new();
            if !config.spatial_hoarding.enabled {
                errors.push(FieldError::new(
                    "spatial_hoarding.enabled",
                    "a supplied cohort needs spatial hoarding enabled",
                ));
            }
            if cohort.len() != config.population as usize {
                errors.push(FieldError::new(
                    "spatial_hoarding.cohort",
                    format!(
                        "expected {} founder slots, got {}",
                        config.population,
                        cohort.len()
                    ),
                ));
            }
            for (slot, traits) in cohort.iter().enumerate() {
                for (name, value) in [("larder", traits.larder), ("defense", traits.defense)] {
                    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
                        errors.push(FieldError::new(
                            format!("spatial_hoarding.cohort.{slot}.{name}"),
                            "must be finite and between 0 and 1",
                        ));
                    }
                }
            }
            if !errors.is_empty() {
                return Err(errors);
            }
        }
        let torus = Torus::new(config.width, config.height);
        let n = config.goods.len();
        if landscapes.len() > n {
            return Err(vec![FieldError::new(
                "landscape",
                format!("{} landscapes for {n} goods", landscapes.len()),
            )]);
        }
        let mut maps = Vec::with_capacity(n);
        for (i, good) in config.goods.iter().enumerate() {
            match landscapes.get(i).and_then(Option::as_ref) {
                Some(c) if c.len() != torus.len() => {
                    return Err(vec![FieldError::new(
                        "landscape",
                        format!(
                            "good {i}: expected {} capacities, got {}",
                            torus.len(),
                            c.len()
                        ),
                    )])
                }
                Some(c) => maps.push(c.clone()),
                None => maps.push(landscape::generate(&good.map, config.width, config.height)),
            }
        }
        let mut walls = vec![0u8; torus.len()];
        for wall in &config.walls {
            let mark = if wall.opaque { 2 } else { 1 };
            for y in wall.y..wall.y.saturating_add(wall.height).min(config.height) {
                for x in wall.x..wall.x.saturating_add(wall.width).min(config.width) {
                    let i = torus.index(Pos::new(x, y));
                    // Opaque wins where rectangles overlap.
                    walls[i] = walls[i].max(mark);
                }
            }
        }
        let sites = (0..torus.len())
            .map(|s| {
                let mut caps = [0.0; MAX_GOODS];
                if walls[s] == 0 {
                    for (slot, map) in caps.iter_mut().zip(&maps) {
                        *slot = map[s];
                    }
                }
                Site::full(&caps[..n])
            })
            .collect();
        let regions = if config.walls.is_empty() {
            Vec::new()
        } else {
            label_regions(torus, &walls)
        };
        let truffles: Vec<Option<u64>> = if config.truffles.share <= 0.0 {
            Vec::new()
        } else {
            (0..torus.len())
                .map(|s| {
                    if walls[s] != 0 {
                        return None;
                    }
                    if rules::truffles::has_spot(s, config.truffles.seed, config.truffles.share) {
                        Some(0u64)
                    } else {
                        None
                    }
                })
                .collect()
        };
        let mut world = World {
            torus,
            tick: 0,
            sites,
            truffles,
            diseases: Vec::new(),
            agents: BTreeMap::new(),
            spatial_cohort: cohort.map(<[FounderTraits]>::to_vec),
            occupancy: vec![None; torus.len()],
            walls,
            regions,
            rng: rng::seeded(seed),
            next_id: 1,
            events: TickEvents::default(),
            stats: Stats::default(),
            loans: BTreeMap::new(),
            next_loan_id: 1,
            config,
            followed: None,
            trail: Vec::new(),
            cache_log: Vec::new(),
            cache_log_full: false,
            probe_dig_at_reserve: false,
            probe_raid_harvests: false,
            record_fates: false,
            cache_open: BTreeMap::new(),
            cache_sites: None,
        };
        if world.config.disease.enabled {
            world.diseases = rules::disease::initial_list(&world.config.disease, &mut world.rng);
        }
        world.populate();
        // Minds 4: `memory.prior: map` gives founders that remember a
        // memory of every non-wall site as the world starts; a no-op
        // otherwise. Runs once here, after placement, so children and
        // replacements (never routed through this) still start empty.
        crate::minds::memory::know_the_map(&mut world);
        world.stats.push(Snapshot::of(&world));
        Ok(world)
    }

    fn populate(&mut self) {
        let n = self.config.population as usize;
        let (w, h) = (self.config.width, self.config.height);
        match self.config.placement {
            Placement::Random => {
                let cells = (0..self.torus.len()).map(|i| self.torus.pos(i)).collect();
                self.place(cells, n, None);
            }
            Placement::Block {
                x,
                y,
                width,
                height,
            } => {
                self.place(rect(x, y, width, height), n, None);
            }
            Placement::Tribes { size } => {
                let blues = n.div_ceil(2);
                self.place(rect(0, h - size, size, size), blues, Some(Tribe::Blue));
                self.place(rect(w - size, 0, size, size), n - blues, Some(Tribe::Red));
            }
        }
    }

    fn place(&mut self, mut cells: Vec<Pos>, n: usize, tribe: Option<Tribe>) {
        if self.has_walls() {
            cells.retain(|&p| !self.is_wall(p));
        }
        cells.shuffle(&mut self.rng);
        for pos in cells.into_iter().take(n) {
            let mut agent = Agent::random(&self.config, pos, self.tick, &mut self.rng);
            if let Some(t) = tribe {
                agent.tags = agent.tags.forced_to(t);
            }
            // Drawn only when asked for, so newborn founders keep every
            // earlier run's random stream.
            if self.config.lifespan.founders == FounderAges::Random {
                agent.age = self.rng.gen_range(0..agent.max_age.max(1));
            }
            rules::disease::endow(self, &mut agent);
            self.insert_agent(agent)
                .expect("placement cells are distinct and empty");
        }
    }

    pub fn agent(&self, id: AgentId) -> Option<&Agent> {
        self.agents.get(&id)
    }

    pub fn agent_mut(&mut self, id: AgentId) -> Option<&mut Agent> {
        self.agents.get_mut(&id)
    }

    /// Living agents in id order.
    pub fn agents(&self) -> impl Iterator<Item = &Agent> {
        self.agents.values()
    }

    /// Living agents in id order, mutably.
    pub(crate) fn agents_mut(&mut self) -> impl Iterator<Item = &mut Agent> {
        self.agents.values_mut()
    }

    pub(crate) fn agent_ids(&self) -> Vec<AgentId> {
        self.agents.keys().copied().collect()
    }

    pub fn population(&self) -> usize {
        self.agents.len()
    }

    pub fn occupant(&self, pos: Pos) -> Option<AgentId> {
        self.occupancy[self.torus.index(pos)]
    }

    pub fn agent_at(&self, pos: Pos) -> Option<&Agent> {
        self.occupant(pos).and_then(|id| self.agents.get(&id))
    }

    pub fn is_occupied(&self, pos: Pos) -> bool {
        self.occupant(pos).is_some() || self.is_wall(pos)
    }

    /// Whether `config.walls` lists any walls.
    pub fn has_walls(&self) -> bool {
        !self.config.walls.is_empty()
    }

    /// Whether `pos` is a wall (fence or opaque): no sugar, no agents.
    pub fn is_wall(&self, pos: Pos) -> bool {
        self.walls[self.torus.index(pos)] != 0
    }

    /// Whether no 4-way path through non-wall sites joins `a` and `b`, by
    /// the components labeled at build. Always false without walls. Other
    /// agents are ignored, so a `true` means every walk from `a` to `b`
    /// fails, never the reverse.
    pub(crate) fn walled_apart(&self, a: Pos, b: Pos) -> bool {
        !self.regions.is_empty()
            && self.regions[self.torus.index(a)] != self.regions[self.torus.index(b)]
    }

    /// Opens the wall at `pos` (it becomes a free, empty site) and relabels
    /// the regions. Only the Minds 5 lab calls this, to open its doorways on
    /// the test evening; everywhere else walls change only on reset. A site
    /// that isn't a wall is left alone.
    pub(crate) fn open_wall(&mut self, pos: Pos) {
        let i = self.torus.index(pos);
        if self.walls[i] == 0 {
            return;
        }
        self.walls[i] = 0;
        self.regions = label_regions(self.torus, &self.walls);
    }

    /// Whether `pos` is an opaque wall: it also stops sight.
    pub fn is_opaque(&self, pos: Pos) -> bool {
        self.walls[self.torus.index(pos)] == 2
    }

    /// Every site visible from `pos` with `vision`: `torus.sight` unchanged
    /// when there are no walls, or stopped at the nearest opaque wall along
    /// each direction.
    pub fn sight(&self, pos: Pos, vision: u32) -> Vec<(Pos, u32)> {
        if !self.has_walls() {
            self.torus.sight(pos, vision)
        } else {
            self.torus.sight_until(pos, vision, |q| self.is_opaque(q))
        }
    }

    pub fn site(&self, pos: Pos) -> &Site {
        &self.sites[self.torus.index(pos)]
    }

    pub fn site_mut(&mut self, pos: Pos) -> &mut Site {
        let i = self.torus.index(pos);
        &mut self.sites[i]
    }

    /// Minds 3: whether `pos` has a truffle spot, and if so whether it's
    /// ripe now. `None` where there's no spot (including everywhere, when
    /// `truffles.share` is 0).
    pub fn truffle(&self, pos: Pos) -> Option<bool> {
        let i = self.torus.index(pos);
        self.truffles
            .get(i)
            .copied()
            .flatten()
            .map(|ripe_at| ripe_at <= self.tick)
    }

    /// Mutable access to the tick a spot at `pos` is next ripe at; `None`
    /// where there's no spot there.
    pub(crate) fn truffle_ripe_at(&mut self, pos: Pos) -> Option<&mut u64> {
        let i = self.torus.index(pos);
        self.truffles.get_mut(i)?.as_mut()
    }

    pub fn empty_sites(&self) -> Vec<Pos> {
        (0..self.torus.len())
            .filter(|&i| self.occupancy[i].is_none() && self.walls[i] == 0)
            .map(|i| self.torus.pos(i))
            .collect()
    }

    /// Good `good`'s capacities, row-major.
    pub fn capacities(&self, good: usize) -> Vec<f64> {
        self.sites.iter().map(|s| s.capacity[good]).collect()
    }

    /// Whether good `good`'s capacities differ from the map its config
    /// generates (painted, or supplied by a share link).
    pub fn landscape_edited(&self, good: usize) -> bool {
        self.config.goods.get(good).is_some_and(|g| {
            let mut generated =
                crate::landscape::generate(&g.map, self.config.width, self.config.height);
            if self.has_walls() {
                for (v, &w) in generated.iter_mut().zip(&self.walls) {
                    if w != 0 {
                        *v = 0.0;
                    }
                }
            }
            self.capacities(good) != generated
        })
    }

    /// Adds `agent` at its position with a fresh id.
    pub fn insert_agent(&mut self, mut agent: Agent) -> Result<AgentId, String> {
        let i = self.torus.index(agent.pos);
        if self.walls[i] != 0 {
            return Err(format!("site ({}, {}) is a wall", agent.pos.x, agent.pos.y));
        }
        if self.occupancy[i].is_some() {
            return Err(format!(
                "site ({}, {}) is occupied",
                agent.pos.x, agent.pos.y
            ));
        }
        let id = self.next_id;
        self.next_id += 1;
        agent.id = id;
        // Minds 5: under `caching.mixed` a founder's rule is dealt by its id.
        if self.config.caching.mixed && agent.parents.is_none() {
            agent.caching_rule = self.config.caching.founder_rule(id);
        }
        // Minds 6: a founder cheats or not by its id, with no draw.
        if self.config.theft.cheaters > 0.0 && agent.parents.is_none() {
            agent.cheater = self.config.theft.founder_cheats(id);
        }
        // Minds 8: so does a founder watch, after its cheater flag (above):
        // `who: hoarders` and `cheaters` deal by that flag. The guard skips
        // only `share` with no watchers, where nobody would watch; the other
        // two ignore `watchers`, so they deal whatever it says.
        if (self.config.watching.watchers > 0.0 || self.config.watching.who != Who::Share)
            && agent.parents.is_none()
        {
            agent.watches = self.config.watching.founder_watches(id, agent.cheater);
        }
        // Minds 5: a central-place forager's home is where it starts life.
        if self.config.central.enabled && agent.home.is_none() {
            agent.home = Some(agent.pos);
        }
        if self.config.spatial_hoarding.enabled {
            let traits = self
                .spatial_cohort
                .as_ref()
                .and_then(|cohort| cohort.get((id - 1) as usize))
                .copied()
                .unwrap_or(FounderTraits {
                    larder: self.config.spatial_hoarding.larder,
                    defense: self.config.spatial_hoarding.defense,
                    cheater: agent.cheater,
                    watches: agent.watches,
                });
            agent.cheater = traits.cheater;
            agent.watches = traits.watches;
            agent.spatial = Some(SpatialState::new(agent.pos, traits));
        }
        self.occupancy[i] = Some(id);
        self.agents.insert(id, agent);
        Ok(id)
    }

    pub(crate) fn move_agent(&mut self, id: AgentId, to: Pos) {
        let from = self.agents[&id].pos;
        if from == to {
            return;
        }
        let (fi, ti) = (self.torus.index(from), self.torus.index(to));
        assert!(self.occupancy[ti].is_none(), "move onto occupied site");
        debug_assert!(!self.is_wall(to), "move onto a wall");
        self.occupancy[fi] = None;
        self.occupancy[ti] = Some(id);
        self.agents.get_mut(&id).expect("live agent").pos = to;
    }

    pub fn events(&self) -> &TickEvents {
        &self.events
    }

    pub fn loans(&self) -> impl Iterator<Item = &Loan> {
        self.loans.values()
    }

    /// Records a loan of `principal` of `good` on the current credit terms
    /// (no transfer).
    pub(crate) fn originate_loan(
        &mut self,
        lender: AgentId,
        borrower: AgentId,
        good: usize,
        principal: f64,
    ) -> LoanId {
        let c = self.config.credit;
        self.originate_loan_on(lender, borrower, good, principal, c.duration, c.rate)
    }

    /// Records a loan of `principal` of `good` for `duration` ticks at `rate`
    /// percent.
    pub(crate) fn originate_loan_on(
        &mut self,
        lender: AgentId,
        borrower: AgentId,
        good: usize,
        principal: f64,
        duration: u32,
        rate: f64,
    ) -> LoanId {
        let id = self.next_loan_id;
        self.next_loan_id += 1;
        let factor = 1.0 + rate / 100.0 * f64::from(duration);
        self.loans.insert(
            id,
            Loan {
                id,
                lender,
                borrower,
                good,
                principal,
                due: principal * factor,
                due_tick: self.tick + u64::from(duration),
                duration,
                rate,
            },
        );
        id
    }

    pub(crate) fn remove_loan(&mut self, id: LoanId) -> Option<Loan> {
        self.loans.remove(&id)
    }

    /// FNV-1a hash of the full dynamic state, for determinism checks.
    pub fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |v: u64| {
            for b in v.to_le_bytes() {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        };
        let n = self.config.goods.len();
        let m = self.config.pollution.pollutants.len();
        let foresight = self.config.foresight.enabled;
        let disease = self.config.disease.enabled;
        let axelrod = self.config.culture.rule == crate::config::CultureKind::Axelrod;
        eat(self.tick);
        for s in &self.sites {
            eat(s.resource[0].to_bits());
            eat(s.capacity[0].to_bits());
            eat(s.pollution[0].to_bits());
            for i in 1..n {
                eat(s.resource[i].to_bits());
                eat(s.capacity[i].to_bits());
            }
            for k in 1..m {
                eat(s.pollution[k].to_bits());
            }
        }
        for a in self.agents.values() {
            eat(a.id);
            eat((u64::from(a.pos.x) << 32) | u64::from(a.pos.y));
            eat(a.holdings[0].to_bits());
            eat(u64::from(a.age));
            eat(a.tags.bits());
            for i in 1..n {
                eat(a.holdings[i].to_bits());
            }
            if foresight {
                eat(u64::from(a.foresight));
            }
            if axelrod {
                eat(a.culture.len() as u64);
                for &t in &a.culture {
                    eat(u64::from(t));
                }
            }
            // Minds 5: caches only when there are any, so every world
            // without caching hashes as before.
            if !a.caches.is_empty() {
                eat(a.caches.len() as u64);
                for (&site, &amount) in &a.caches {
                    eat(u64::from(site));
                    eat(amount.to_bits());
                }
            }
            // Minds 6: a stomach only when there's something in it.
            if a.fed != 0.0 {
                eat(a.fed.to_bits());
            }
            if disease {
                eat(u64::from(a.immune.len()));
                eat(a.immune.bits());
                eat(u64::from(a.immune_genome.len()));
                eat(a.immune_genome.bits());
                eat(a.diseases.len() as u64);
                for &d in &a.diseases {
                    eat(u64::from(d));
                }
            }
        }
        if disease {
            for d in &self.diseases {
                eat(u64::from(d.len()));
                eat(d.bits());
            }
        }
        for l in self.loans.values() {
            eat(l.id);
            eat(l.due.to_bits());
            eat(l.due_tick);
            eat(u64::from(l.duration));
            eat(l.rate.to_bits());
            if n >= 2 {
                eat(l.good as u64);
            }
        }
        h
    }

    /// Takes an agent off the grid with no death event and no inheritance.
    /// Every removal (every cause of death, and an edit) passes here, so a
    /// Minds 5 agent's caches are counted into `events.cache_lost` once and
    /// leave the world with it. An edit between ticks removes caches too,
    /// but that count lands in the finished tick's events after its
    /// statistics were taken, and the next tick resets them: caches removed
    /// by an edit aren't reported in any tick's `cache_lost`. Minds 6: its
    /// open fate records close as `Lost`.
    pub(crate) fn remove(&mut self, id: AgentId) -> Option<Agent> {
        let agent = self.agents.remove(&id)?;
        if !agent.caches.is_empty() {
            self.events.cache_lost += agent.caches.values().sum::<f64>();
        }
        if self.config.spatial_hoarding.enabled {
            use crate::minds::spatial_hoarding::{state::StoreKind, stores};
            let scatter_lost = agent.caches.values().sum::<f64>();
            stores::events(self, StoreKind::Scatter)
                .expect("enabled")
                .lost += scatter_lost;
            if let Some(s) = &agent.spatial {
                self.events.cache_lost += s.larder;
                stores::events(self, StoreKind::Larder)
                    .expect("enabled")
                    .lost += s.larder;
                crate::minds::caching::fates::lose_larder(
                    self,
                    id,
                    self.torus.index(s.home) as u32,
                    s.larder,
                    s.larder_since.unwrap_or(self.tick),
                );
            }
        }
        self.events.fed_lost += agent.fed;
        crate::minds::caching::fates::close_lost(self, id, &agent.caches, &agent.cache_since);
        if self.cache_sites.is_some() {
            for &site in agent.caches.keys() {
                crate::minds::caching::theft::note(self, id, site, false);
            }
        }
        let i = self.torus.index(agent.pos);
        self.occupancy[i] = None;
        if !self.loans.is_empty() {
            self.loans.retain(|_, l| l.lender != id && l.borrower != id);
        }
        Some(agent)
    }

    /// Removes an agent from play and records its death. With rule I on, its
    /// remaining sugar is split equally among its living children. A dead
    /// lender's outstanding claims pass to its living children as well.
    pub(crate) fn kill(&mut self, id: AgentId, cause: DeathCause) -> Option<Agent> {
        let claims: Vec<Loan> = if self.config.inheritance.enabled {
            self.loans
                .values()
                .filter(|l| l.lender == id)
                .copied()
                .collect()
        } else {
            Vec::new()
        };
        let agent = self.remove(id)?;
        self.events.deaths.push(Death {
            id,
            tribe: agent.tribe(),
            cause,
        });
        if self.config.inheritance.enabled {
            self.bequeath(&agent);
        }
        if !claims.is_empty() {
            self.pass_on_claims(&agent, claims);
        }
        Some(agent)
    }

    fn bequeath(&mut self, agent: &Agent) {
        let heirs: Vec<AgentId> = agent
            .children
            .iter()
            .copied()
            .filter(|c| self.agents.contains_key(c))
            .collect();
        let n = self.config.goods.len();
        let count = heirs.len() as f64;
        let shares: [f64; MAX_GOODS] = std::array::from_fn(|i| {
            if i < n && agent.holdings[i] > 0.0 {
                agent.holdings[i] / count
            } else {
                0.0
            }
        });
        if heirs.is_empty() || shares.iter().all(|&s| s == 0.0) {
            return;
        }
        for heir in heirs {
            let h = self.agents.get_mut(&heir).expect("living heir");
            for (have, share) in h.holdings.iter_mut().zip(&shares).take(n) {
                *have += share;
            }
        }
    }

    /// Splits a dead lender's claims equally among its living children; a
    /// child who is the borrower has its own share forgiven.
    fn pass_on_claims(&mut self, lender: &Agent, claims: Vec<Loan>) {
        let heirs: Vec<AgentId> = lender
            .children
            .iter()
            .copied()
            .filter(|c| self.agents.contains_key(c))
            .collect();
        if heirs.is_empty() {
            return;
        }
        let n = heirs.len() as f64;
        for claim in claims {
            if !self.agents.contains_key(&claim.borrower) {
                continue;
            }
            for &heir in &heirs {
                if heir == claim.borrower {
                    // A claim on oneself is forgiven.
                    continue;
                }
                let id = self.next_loan_id;
                self.next_loan_id += 1;
                self.loans.insert(
                    id,
                    Loan {
                        id,
                        lender: heir,
                        principal: claim.principal / n,
                        due: claim.due / n,
                        ..claim
                    },
                );
            }
        }
    }

    /// One tick: every living agent takes a turn in a fresh random order
    /// (agents born or killed during the tick are skipped), then the
    /// environment updates and everyone ages.
    pub fn step(&mut self) {
        self.events = TickEvents::default();
        self.apply_schedule();
        if self.config.pilfering_on() {
            crate::minds::caching::theft::count_candidates(self);
        }
        // Minds 8: forget seen caches older than `span`, before anyone moves.
        if self.config.watching.on {
            crate::minds::caching::watching::sweep(self);
        }
        // Minds 5: a lab world applies its protocol's day (placement, food,
        // doorways, the test evening's burying) before anyone moves.
        if self.config.lab.is_some() {
            crate::minds::caching::lab::apply(self);
        }
        if self.config.disease.enabled {
            rules::disease::outbreaks(self);
        }
        let mut order = self.agent_ids();
        order.shuffle(&mut self.rng);
        for id in order {
            if self.agents.contains_key(&id) {
                rules::agent_turn(self, id);
            }
        }
        if !self.loans.is_empty() {
            rules::credit::settle(self);
        }
        rules::growback::apply(self);
        rules::pollution::diffuse(self);
        rules::replacement::apply(self);
        for agent in self.agents.values_mut() {
            agent.age += 1;
        }
        self.tick += 1;
        let snapshot = Snapshot::of(self);
        self.stats.push(snapshot);
        self.record_trail();
    }

    pub fn run(&mut self, ticks: u32) {
        for _ in 0..ticks {
            if self.is_finished() {
                break;
            }
            self.step();
        }
    }

    /// Whether the run has stopped: Axelrod's culture rule at work (K on)
    /// with `stop_when_settled`, at least two agents, and the latest tick
    /// settled (milestone 14).
    pub fn is_finished(&self) -> bool {
        self.config.culture.stop_when_settled
            && self.config.culture.axelrod()
            && self.population() >= 2
            && self
                .stats
                .latest()
                .and_then(|s| s.axelrod)
                .is_some_and(|a| a.settled)
    }

    /// Follows agent `id` from now on (a fresh trail that records its current
    /// position at once, then its position at the end of every tick), or
    /// stops following with `None` (the trail is cleared). A followed agent
    /// that dies leaves its trail as it was.
    pub fn follow(&mut self, id: Option<AgentId>) {
        self.followed = id;
        self.trail.clear();
        self.record_trail();
    }

    pub fn followed(&self) -> Option<AgentId> {
        self.followed
    }

    /// The followed agent's positions, oldest first (at most `TRAIL_LEN`).
    pub fn trail(&self) -> &[Pos] {
        &self.trail
    }

    fn record_trail(&mut self) {
        let Some(pos) = self
            .followed
            .and_then(|id| self.agents.get(&id))
            .map(|a| a.pos)
        else {
            return;
        };
        if self.trail.len() == TRAIL_LEN {
            self.trail.remove(0);
        }
        self.trail.push(pos);
    }

    /// Applies scheduled changes due at the tick about to run. Entries not yet
    /// fired were validated against the config that reaches them (`World::new`
    /// checks the whole schedule; `set_config` checks entries with
    /// `tick >= self.tick`), so failures should not happen; any are ignored.
    fn apply_schedule(&mut self) {
        let due: Vec<_> = self
            .config
            .schedule
            .iter()
            .filter(|c| c.tick == self.tick)
            .cloned()
            .collect();
        for change in due {
            if let Ok(next) = self.config.apply_change(&change) {
                self.config = next;
            }
        }
    }
}

/// Labels the connected components of the non-wall sites (4-way, on the
/// torus) by flood fill, in index order; walls get `u32::MAX`.
fn label_regions(torus: Torus, walls: &[u8]) -> Vec<u32> {
    let mut regions = vec![u32::MAX; walls.len()];
    let mut next = 0;
    let mut stack = Vec::new();
    for start in 0..walls.len() {
        if walls[start] != 0 || regions[start] != u32::MAX {
            continue;
        }
        regions[start] = next;
        stack.push(start);
        while let Some(i) = stack.pop() {
            for q in torus.neighbors(torus.pos(i)) {
                let j = torus.index(q);
                if walls[j] == 0 && regions[j] == u32::MAX {
                    regions[j] = next;
                    stack.push(j);
                }
            }
        }
        next += 1;
    }
    regions
}

fn rect(x: u32, y: u32, width: u32, height: u32) -> Vec<Pos> {
    (y..y + height)
        .flat_map(|yy| (x..x + width).map(move |xx| Pos::new(xx, yy)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn founders_can_start_at_random_ages_without_moving_anyone_elses_draws() {
        let mut c = crate::presets::by_id("iii-2-sex").unwrap().config;
        let newborn = World::new(c.clone(), 3).unwrap();
        assert!(newborn.agents().all(|a| a.age == 0));
        c.lifespan.founders = FounderAges::Random;
        let aged = World::new(c, 3).unwrap();
        assert!(aged.agents().all(|a| a.age < a.max_age));
        assert!(aged.agents().filter(|a| a.age > 20).count() > aged.population() / 2);
    }
    use crate::config::{Config, Good, Placement};

    #[test]
    fn default_world_places_400_agents_on_distinct_sites() {
        let w = World::new(Config::default(), 1).unwrap();
        assert_eq!(w.population(), 400);
        let mut positions: Vec<Pos> = w.agents().map(|a| a.pos).collect();
        positions.sort();
        positions.dedup();
        assert_eq!(positions.len(), 400);
        for a in w.agents() {
            assert_eq!(w.occupant(a.pos), Some(a.id));
            assert!((1..=6).contains(&a.vision));
            assert!((1..=4).contains(&a.metabolism[0]));
            assert!((5.0..=25.0).contains(&a.holdings[0]));
            assert_eq!(a.holdings[0], a.initial[0]);
        }
        assert_eq!(
            w.site(Pos::new(37, 5)).resource[0],
            4.0,
            "sugar starts at capacity"
        );
    }

    #[test]
    fn tribes_placement_puts_blues_southwest_and_reds_northeast() {
        let c = Config {
            placement: Placement::Tribes { size: 20 },
            ..Config::default()
        };
        let w = World::new(c, 3).unwrap();
        assert_eq!(w.population(), 400);
        for a in w.agents() {
            match a.tribe() {
                Tribe::Blue => assert!(a.pos.x < 20 && a.pos.y >= 30),
                Tribe::Red => assert!(a.pos.x >= 30 && a.pos.y < 20),
            }
        }
    }

    #[test]
    fn same_seed_same_world_different_seed_different_world() {
        let a = World::new(Config::default(), 9).unwrap();
        let b = World::new(Config::default(), 9).unwrap();
        let c = World::new(Config::default(), 10).unwrap();
        assert_eq!(a.fingerprint(), b.fingerprint());
        assert_ne!(a.fingerprint(), c.fingerprint());
    }

    #[test]
    fn custom_capacities_must_match_grid() {
        let err = World::with_capacities(Config::default(), 1, Some(&[1.0; 10]))
            .err()
            .unwrap();
        assert_eq!(err[0].field, "landscape");
        let w = World::with_capacities(Config::default(), 1, Some(&[2.0; 2500])).unwrap();
        assert!(w.landscape_edited(0));
        assert_eq!(w.site(Pos::new(0, 0)).capacity[0], 2.0);
    }

    #[test]
    fn each_good_has_its_own_supplied_or_generated_landscape() {
        let mut c = Config::default();
        c.add_good(Good::spice());
        let spice = vec![2.0; 2500];
        let w = World::with_landscapes(c.clone(), 1, &[None, Some(spice.clone())]).unwrap();
        assert_eq!(w.capacities(1), spice);
        assert_eq!(
            w.capacities(0),
            landscape::generate(&c.goods[0].map, 50, 50)
        );
        assert!(!w.landscape_edited(0) && w.landscape_edited(1));
        let too_many = World::with_landscapes(c.clone(), 1, &[None, None, None]);
        assert_eq!(too_many.err().unwrap()[0].field, "landscape");
        let short = World::with_landscapes(c, 1, &[Some(vec![1.0; 3])]);
        assert_eq!(short.err().unwrap()[0].field, "landscape");
    }

    #[test]
    fn invalid_config_is_rejected() {
        let c = Config {
            population: 10_000,
            ..Config::default()
        };
        assert!(World::new(c, 1).is_err());
    }

    #[test]
    fn insert_rejects_occupied_site() {
        let mut w = crate::testkit::blank_world(5, 5);
        crate::testkit::spawn(&mut w, 2, 2);
        let clone = w.agent_at(Pos::new(2, 2)).unwrap().clone();
        assert!(w.insert_agent(clone).is_err());
    }

    #[test]
    fn step_is_deterministic_for_a_seed() {
        let mut a = World::new(Config::default(), 42).unwrap();
        let mut b = World::new(Config::default(), 42).unwrap();
        a.run(50);
        b.run(50);
        assert_eq!(a.tick, 50);
        assert_eq!(a.fingerprint(), b.fingerprint());
    }

    #[test]
    fn population_declines_toward_carrying_capacity() {
        let mut w = World::new(Config::default(), 5).unwrap();
        w.run(100);
        assert!(
            w.population() < 400 && w.population() > 100,
            "got {}",
            w.population()
        );
        for a in w.agents() {
            assert!(a.holdings[0] > 0.0);
            assert_eq!(a.age, 100, "immortal first generation ages every tick");
        }
    }

    #[test]
    fn scheduled_changes_apply_when_their_tick_is_reached() {
        use crate::config::ScheduledChange;
        let mut c = crate::testkit::blank_config(10, 10);
        c.schedule = vec![ScheduledChange {
            tick: 2,
            set: [("pollution.enabled".to_string(), serde_json::json!(true))]
                .into_iter()
                .collect(),
        }];
        let mut w = World::new(c, 1).unwrap();
        w.step(); // tick 0 → 1
        assert!(!w.config.pollution.enabled);
        w.step(); // tick 1 → 2
        assert!(!w.config.pollution.enabled);
        w.step(); // starts at tick 2: applied
        assert!(w.config.pollution.enabled);
        assert_eq!(w.config.schedule.len(), 1, "the schedule itself is kept");
    }

    #[test]
    fn disease_worlds_draw_a_list_and_endow_agents() {
        let mut c = Config::default();
        c.disease.enabled = true;
        let w = World::new(c, 1).unwrap();
        assert_eq!(w.diseases.len(), 10);
        assert!(w.diseases.iter().all(|d| (1..=10).contains(&d.len())));
        let mut carried = 0;
        for a in w.agents() {
            assert_eq!(a.immune.len(), 50);
            assert!(a.diseases.len() <= 4);
            let mut ids = a.diseases.clone();
            ids.sort_unstable();
            ids.dedup();
            assert_eq!(ids.len(), a.diseases.len(), "distinct diseases");
            for &d in &a.diseases {
                assert!(
                    !a.immune.contains(&w.diseases[d as usize]),
                    "never starts with a disease it is immune to"
                );
            }
            carried += a.diseases.len();
        }
        assert!(carried > 0, "some agents start sick");
        assert!(World::new(Config::default(), 1)
            .unwrap()
            .diseases
            .is_empty());
    }

    #[test]
    fn fingerprint_covers_disease_state_only_when_disease_is_on() {
        let mut w = crate::testkit::blank_world(5, 5);
        let id = crate::testkit::spawn(&mut w, 1, 1);
        let before = w.fingerprint();
        w.agent_mut(id).unwrap().diseases.push(0);
        w.agent_mut(id).unwrap().immune.flip(0);
        assert_eq!(w.fingerprint(), before, "ignored while disease is off");
        w.config.disease.enabled = true;
        let on = w.fingerprint();
        w.agent_mut(id).unwrap().immune.flip(1);
        assert_ne!(w.fingerprint(), on, "immune strings are hashed");
        let on = w.fingerprint();
        w.diseases.push(crate::bits::Bits::parse("101").unwrap());
        assert_ne!(w.fingerprint(), on, "the disease list is hashed");
    }

    #[test]
    fn goods_are_stored_in_per_good_slots() {
        let mut c = Config::default();
        c.add_good(Good::spice());
        let w = World::new(c, 1).unwrap();
        assert_eq!(
            w.site(Pos::new(37, 5)).capacity[0],
            4.0,
            "sugar's northeast peak"
        );
        assert_eq!(
            w.site(Pos::new(12, 5)).capacity[1],
            4.0,
            "spice's northwest peak"
        );
        assert!(w
            .sites
            .iter()
            .all(|s| s.resource[2..].iter().all(|&x| x == 0.0)));
        for a in w.agents() {
            assert_eq!(a.holdings[..2], a.initial[..2]);
            assert!((1..=4).contains(&a.metabolism[1]));
            assert!(a.holdings[2..].iter().all(|&x| x == 0.0));
        }
    }

    #[test]
    fn three_goods_and_two_pollutants_validate_and_run() {
        use crate::config::{Good, Map, Pollutant, Transform, URange};
        let mut c = Config::default();
        c.add_good(Good::spice());
        c.add_good(Good {
            name: "salt".into(),
            color: "#7fb3d5".into(),
            map: Map::TwoPeaks {
                transform: Transform::Rotate90,
            },
            ..Good::sugar()
        });
        // Light metabolisms and generous endowments so a three-good
        // population is certain to survive 20 ticks.
        for g in &mut c.goods {
            g.metabolism = URange::new(1, 2);
            g.endowment = URange::new(25, 50);
        }
        c.trade.enabled = true;
        c.pollution.enabled = true;
        c.pollution.pollutants.push(Pollutant {
            name: "runoff".into(),
            production: vec![0.0, 0.0, 1.0],
            consumption: vec![0.0; 3],
            devalues: vec![false, false, true],
        });
        let mut w = World::new(c, 1).unwrap();
        w.run(20);
        assert!(w.population() > 0);
        assert_eq!(
            w.site(Pos::new(44, 37)).capacity[2],
            4.0,
            "salt's south-east peak"
        );
        assert!(w.sites.iter().any(|s| s.pollution[1] > 0.0), "runoff forms");
        let before = w.fingerprint();
        w.sites[0].resource[2] += 1.0;
        assert_ne!(w.fingerprint(), before, "good 2 is hashed");
        let before = w.fingerprint();
        w.sites[0].pollution[1] += 1.0;
        assert_ne!(w.fingerprint(), before, "pollutant 1 is hashed");
    }

    #[test]
    fn a_followed_agent_leaves_a_capped_trail_that_is_not_hashed() {
        let mut w = crate::testkit::blank_world(10, 10);
        let id = crate::testkit::spawn(&mut w, 2, 3);
        let mut twin = crate::testkit::blank_world(10, 10);
        crate::testkit::spawn(&mut twin, 2, 3);
        assert_eq!((w.followed(), w.trail().len()), (None, 0));
        w.follow(Some(id));
        assert_eq!(w.followed(), Some(id));
        assert_eq!(w.trail(), &[Pos::new(2, 3)], "the current position at once");
        w.run(3);
        twin.run(3);
        assert_eq!(w.trail().len(), 4);
        assert_eq!(*w.trail().last().unwrap(), w.agent(id).unwrap().pos);
        assert_eq!(
            w.fingerprint(),
            twin.fingerprint(),
            "trails are not simulation state"
        );
        w.run(TRAIL_LEN as u32);
        assert_eq!(
            w.trail().len(),
            TRAIL_LEN,
            "the oldest positions are dropped"
        );
        assert_eq!(*w.trail().last().unwrap(), w.agent(id).unwrap().pos);
        // Death: the trail stops growing and stays until `follow` is called.
        let before = w.trail().to_vec();
        let pos = w.agent(id).unwrap().pos;
        w.remove_agent(pos.x, pos.y).unwrap();
        w.run(2);
        assert_eq!(w.trail(), &before[..]);
        assert_eq!(w.followed(), Some(id));
        w.follow(None);
        assert_eq!((w.followed(), w.trail().len()), (None, 0));
        w.follow(Some(999));
        assert!(w.trail().is_empty(), "nobody alive to record");
    }

    fn walled(walls: Vec<crate::config::Wall>) -> World {
        let mut c = crate::testkit::blank_config(11, 11);
        c.walls = walls;
        World::new(c, 7).unwrap()
    }
    fn wall(x: u32, y: u32, width: u32, height: u32, opaque: bool) -> crate::config::Wall {
        crate::config::Wall {
            x,
            y,
            width,
            height,
            opaque,
        }
    }

    #[test]
    fn with_no_walls_sight_is_the_toruss() {
        let w = walled(vec![]);
        for v in [1, 3, 5, 6, 10] {
            assert_eq!(w.sight(Pos::new(5, 5), v), w.torus.sight(Pos::new(5, 5), v));
            assert_eq!(
                w.sight(Pos::new(0, 10), v),
                w.torus.sight(Pos::new(0, 10), v)
            );
        }
        assert!(!w.has_walls());
    }

    #[test]
    fn opaque_walls_stop_sight_and_fences_do_not() {
        let w = walled(vec![wall(5, 2, 1, 1, true), wall(7, 5, 1, 1, false)]);
        let seen: Vec<Pos> = w
            .sight(Pos::new(5, 5), 4)
            .into_iter()
            .map(|s| s.0)
            .collect();
        assert!(seen.contains(&Pos::new(5, 3)));
        assert!(
            !seen.contains(&Pos::new(5, 2)),
            "the wall itself isn't a sight line site"
        );
        assert!(!seen.contains(&Pos::new(5, 1)), "behind the wall");
        assert!(
            seen.contains(&Pos::new(7, 5)) && seen.contains(&Pos::new(8, 5)),
            "a fence doesn't block sight"
        );
    }

    #[test]
    fn walls_hold_nothing_and_nobody() {
        let mut w = walled(vec![wall(3, 3, 2, 2, false)]);
        let p = Pos::new(3, 3);
        assert!(w.is_wall(p) && w.is_occupied(p) && w.occupant(p).is_none());
        assert_eq!((w.site(p).capacity[0], w.site(p).resource[0]), (0.0, 0.0));
        assert!(!w.empty_sites().contains(&p));
        let mut a = crate::testkit::agent_at(&w, 3, 3);
        a.pos = p;
        assert!(w.insert_agent(a).unwrap_err().contains("wall"));
        crate::rules::growback::apply(&mut w);
        assert_eq!(w.site(p).resource[0], 0.0);
    }

    #[test]
    fn placement_skips_walls() {
        let c = crate::config::Config {
            walls: vec![wall(0, 0, 50, 40, true)],
            population: 400,
            ..crate::config::Config::default()
        };
        let w = World::new(c, 3).unwrap();
        assert_eq!(w.population(), 400);
        assert!(w.agents().all(|a| a.pos.y >= 40));
    }

    // --- Minds 3: truffles ---

    fn truffle_config(width: u32, height: u32, share: f64) -> crate::config::Config {
        let mut c = crate::testkit::blank_config(width, height);
        c.truffles.share = share;
        c
    }

    fn all_positions(w: &World) -> Vec<Pos> {
        (0..w.torus.len()).map(|i| w.torus.pos(i)).collect()
    }

    #[test]
    fn truffle_layout_is_the_same_across_world_seeds() {
        let mut c = truffle_config(20, 20, 0.2);
        c.truffles.seed = 3;
        let a = World::new(c.clone(), 1).unwrap();
        let b = World::new(c, 2).unwrap();
        for p in all_positions(&a) {
            assert_eq!(a.truffle(p), b.truffle(p), "at {p:?}");
        }
        // At least the hash actually placed something, so the check above
        // isn't vacuously true.
        assert!(all_positions(&a).iter().any(|&p| a.truffle(p).is_some()));
    }

    #[test]
    fn truffle_share_is_within_one_percent_over_a_100_by_100_grid() {
        let c = truffle_config(100, 100, 0.05);
        let w = World::new(c, 1).unwrap();
        let n = 100 * 100;
        let count = all_positions(&w)
            .iter()
            .filter(|&&p| w.truffle(p).is_some())
            .count();
        let share = count as f64 / f64::from(n);
        assert!(
            (share - 0.05).abs() < 0.0005,
            "share {share} within 1% of 0.05"
        );
    }

    #[test]
    fn truffle_share_zero_gives_none() {
        let c = truffle_config(10, 10, 0.0);
        let w = World::new(c, 1).unwrap();
        assert!(all_positions(&w).iter().all(|&p| w.truffle(p).is_none()));
    }

    #[test]
    fn truffle_share_one_gives_every_non_wall_site_a_ripe_spot_and_none_on_walls() {
        let mut c = truffle_config(11, 11, 1.0);
        c.walls = vec![wall(3, 3, 2, 2, true)];
        let w = World::new(c, 1).unwrap();
        for p in all_positions(&w) {
            if w.is_wall(p) {
                assert_eq!(w.truffle(p), None, "no spot on a wall, at {p:?}");
            } else {
                assert_eq!(
                    w.truffle(p),
                    Some(true),
                    "every non-wall site has a ripe spot, at {p:?}"
                );
            }
        }
    }
}
