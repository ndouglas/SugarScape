//! Researcher-only physical connectivity, never worker navigation input.
use super::{
    food::{FoodLedger, FoodState},
    metrics::AccessCompute,
    setup::neighbors,
    terrain::Terrain,
    Checked, Pos, Setup,
};
use std::collections::{BTreeSet, VecDeque};
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct EventContext {
    pub tick: u32,
    pub opportunity: u64,
    pub worker: u32,
    pub excavated: u32,
    pub spoil_disposed: u32,
    pub food_delivered: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct EventMilestone {
    pub context: EventContext,
    pub pos: Pos,
    pub nest_distance: u32,
}
// Staged Task 4 observer/world and Task 5 researcher-view consumers.
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub(super) struct FoodAccessRecord {
    pub(super) id: u64,
    pub(super) initially_exposed: bool,
    pub(super) initially_accessible: bool,
    pub(super) first_exposure: Option<EventMilestone>,
    pub(super) first_access: Option<EventMilestone>,
    pub(super) accessible: bool,
    pub(super) distance: Option<u32>,
}
// Staged Task 4 observer/world and Task 5 researcher-view consumers.
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub(super) struct AccessSummary {
    pub(super) initially_exposed: u32,
    pub(super) initially_accessible: u32,
    pub(super) accessible: u32,
    pub(super) records: Vec<FoodAccessRecord>,
    pub(super) compute: AccessCompute,
}
// Staged Task 4 observer/world and Task 5 researcher-view consumers.
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct AccessObserver {
    width: u32,
    height: u32,
    workers: u32,
    seen_digs: u32,
    last_opportunity: u64,
    records: Vec<FoodAccessRecord>,
    distances: Vec<Option<u32>>,
    observed_open: Vec<bool>,
    compute: AccessCompute,
}
// Staged Task 4 observer/world and Task 5 researcher-view consumers.
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct AccessDelta {
    pub(super) exposure: Option<EventMilestone>,
    pub(super) access: Option<EventMilestone>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct Milestones {
    pub first_excavation: Option<EventMilestone>,
    pub first_exposure: Option<EventMilestone>,
    pub first_access: Option<EventMilestone>,
    pub first_disposal: Option<EventMilestone>,
    pub first_pickup_tick: Option<u32>,
    pub first_delivery_tick: Option<u32>,
    pub all_food_delivered_tick: Option<u32>,
}
// Staged Task 4 transactional world observation and Task 5 output consumers.
#[allow(dead_code)]
impl AccessObserver {
    pub(super) fn new(setup: &Setup, terrain: &Terrain, food: &FoodLedger) -> Checked<Self> {
        setup.validate()?;
        if terrain.counts().excavated != 0
            || food.inventory().carried != 0
            || food.inventory().delivered != 0
        {
            return Err(error(
                "access.initial",
                "observer requires unexcavated terrain and uncarried, undelivered initial food",
            ));
        }
        let mut observer = Self {
            width: setup.width,
            height: setup.height,
            workers: setup.workers.len() as u32,
            seen_digs: 0,
            last_opportunity: 0,
            records: vec![],
            distances: vec![],
            observed_open: vec![],
            compute: AccessCompute::default(),
        };
        let open = observer.check_inputs(setup, terrain, food)?;
        let (distances, compute) = observer.bfs(setup, &open)?;
        observer.distances = distances;
        observer.observed_open = open;
        observer.compute = compute;
        for view in food.views() {
            let distance = observer.distance(view.resource.pos)?;
            observer.records.push(FoodAccessRecord {
                id: view.resource.id,
                initially_exposed: view.state == FoodState::Available,
                initially_accessible: distance.is_some(),
                first_exposure: None,
                first_access: None,
                accessible: distance.is_some(),
                distance,
            });
        }
        observer.check(setup, terrain, food)?;
        Ok(observer)
    }
    pub(super) fn after_dig(
        &mut self,
        setup: &Setup,
        terrain: &Terrain,
        food: &FoodLedger,
        context: &EventContext,
    ) -> Checked<AccessDelta> {
        // Validate the previous graph before computing a replacement; corrupt caches are never repaired silently.
        self.check_basic()?;
        let open = self.check_inputs(setup, terrain, food)?;
        self.check_graph(setup)?;
        self.check_records(setup, food)?;
        let seen = self
            .seen_digs
            .checked_add(1)
            .ok_or_else(|| error("access.seen_digs", "counter overflow"))?;
        self.check_context(context, seen)?;
        if terrain.counts().excavated != seen
            || context.opportunity <= self.last_opportunity
            || context.food_delivered != food.inventory().delivered
            || context.spoil_disposed >= seen
        {
            return Err(error("access.context","update requires one new excavation, increasing opportunity, matching food delivery count and its new spoil still carried"));
        }
        let mut opened = None;
        let mask: BTreeSet<_> = setup.diggable.iter().copied().collect();
        for (index, (&old, &new)) in self.observed_open.iter().zip(&open).enumerate() {
            if old == new {
                continue;
            }
            let pos = self.position(index);
            if old
                || !new
                || opened.is_some()
                || !mask.contains(&pos)
                || !terrain.was_excavated(pos)?
            {
                return Err(error(
                    "access.terrain",
                    "update requires exactly one eligible solid-to-open change",
                ));
            }
            opened = Some(pos);
        }
        let opened = opened.ok_or_else(|| error("access.terrain", "missing new excavated cell"))?;
        let mut pending = self.clone();
        let (distances, compute) = self.bfs(setup, &open)?;
        pending.compute.checked_include(&compute)?;
        pending.distances = distances;
        pending.observed_open = open;
        pending.seen_digs = seen;
        pending.last_opportunity = context.opportunity;
        // A legal worker dig extends the nest-connected component; disconnected primitive digs are not committed events.
        pending.milestone(context, opened)?;
        let mut delta = AccessDelta {
            exposure: None,
            access: None,
        };
        for (index, view) in food.views().iter().enumerate() {
            let distance = pending.distance(view.resource.pos)?;
            let record = &pending.records[index];
            let expose = !record.initially_exposed
                && record.first_exposure.is_none()
                && view.state != FoodState::Hidden;
            let access =
                !record.initially_accessible && record.first_access.is_none() && distance.is_some();
            let event = if expose || access {
                Some(pending.milestone(context, view.resource.pos)?)
            } else {
                None
            };
            let record = &mut pending.records[index];
            if expose {
                record.first_exposure = event.clone();
                if delta.exposure.is_none() {
                    delta.exposure = event.clone();
                }
            }
            if access {
                record.first_access = event.clone();
                if delta.access.is_none() {
                    delta.access = event;
                }
            }
            record.distance = distance;
            record.accessible = distance.is_some();
        }
        pending.check(setup, terrain, food)?;
        *self = pending;
        Ok(delta)
    }
    pub(super) fn milestone(&self, context: &EventContext, pos: Pos) -> Checked<EventMilestone> {
        self.check_context(context, self.seen_digs)?;
        if context.opportunity < self.last_opportunity {
            return Err(error(
                "access.context",
                "milestone precedes the current observed dig",
            ));
        }
        let nest_distance = self.distance(pos)?.ok_or_else(|| {
            error(
                "access.milestone",
                "position must have a current physical route to the nest",
            )
        })?;
        Ok(EventMilestone {
            context: context.clone(),
            pos,
            nest_distance,
        })
    }
    pub(super) fn summary(&self) -> Checked<AccessSummary> {
        self.check_basic()?;
        if self.records.len() > 256 {
            return Err(error("access.records", "at most 256 food records"));
        }
        Ok(AccessSummary {
            initially_exposed: self.records.iter().filter(|r| r.initially_exposed).count() as u32,
            initially_accessible: self
                .records
                .iter()
                .filter(|r| r.initially_accessible)
                .count() as u32,
            accessible: self.records.iter().filter(|r| r.accessible).count() as u32,
            records: self.records.clone(),
            compute: self.compute,
        })
    }
    pub(super) fn distance(&self, pos: Pos) -> Checked<Option<u32>> {
        self.distances
            .get(self.index(pos)?)
            .copied()
            .ok_or_else(|| error("access.distances", "cache dimensions disagree"))
    }
    pub(super) fn check(&self, setup: &Setup, terrain: &Terrain, food: &FoodLedger) -> Checked<()> {
        self.check_basic()?;
        let open = self.check_inputs(setup, terrain, food)?;
        if open != self.observed_open || terrain.counts().excavated != self.seen_digs {
            return Err(error(
                "access.terrain",
                "observed graph and excavation provenance must match terrain",
            ));
        }
        self.check_graph(setup)?;
        self.check_records(setup, food)
    }
    fn index(&self, pos: Pos) -> Checked<usize> {
        if pos.x >= self.width || pos.y >= self.height {
            return Err(error("access.pos", "must be inside the observed grid"));
        }
        Ok((pos.y * self.width + pos.x) as usize)
    }
    fn position(&self, index: usize) -> Pos {
        Pos {
            x: index as u32 % self.width,
            y: index as u32 / self.width,
        }
    }
    fn check_context(&self, context: &EventContext, excavated: u32) -> Checked<()> {
        if !(1..=256).contains(&self.workers)
            || context.opportunity == 0
            || context.opportunity > 1_000_000
        {
            return Err(error(
                "access.context",
                "requires a positive bounded opportunity and valid population",
            ));
        }
        let tick = (context.opportunity - 1) / u64::from(self.workers);
        let complete = (tick + 1) * u64::from(self.workers);
        if tick >= 7200
            || complete > 1_000_000
            || u64::from(context.tick) != tick
            || u64::from(context.worker) != (context.opportunity - 1) % u64::from(self.workers)
            || context.excavated != excavated
            || u64::from(excavated) > context.opportunity
            || context.spoil_disposed > excavated
            || context.food_delivered > self.records.len() as u32
        {
            return Err(error(
                "access.context",
                "tick/worker, complete-tick budget and material provenance disagree",
            ));
        }
        Ok(())
    }
    fn check_basic(&self) -> Checked<()> {
        super::setup::dimensions(self.width, self.height)?;
        let size = u64::from(self.width) * u64::from(self.height);
        let calls = u64::from(self.seen_digs) + 1;
        if !(1..=256).contains(&self.workers)
            || self.distances.len() != size as usize
            || self.observed_open.len() != size as usize
            || u64::from(self.seen_digs) > size
            || self.compute.calls != calls
            || self.compute.visits < calls * 2
            || self.compute.visits > calls * size
            || self.compute.peak_queue < 2
            || self.compute.peak_queue > size
            || self.last_opportunity < u64::from(self.seen_digs)
            || (self.seen_digs == 0) != (self.last_opportunity == 0)
        {
            return Err(error(
                "access.cache",
                "dimensions, call counts or bounded observer provenance disagree",
            ));
        }
        if self.last_opportunity > 0 {
            let tick = (self.last_opportunity - 1) / u64::from(self.workers);
            if tick >= 7200 || (tick + 1) * u64::from(self.workers) > 1_000_000 {
                return Err(error(
                    "access.last_opportunity",
                    "outside complete-tick horizon",
                ));
            }
        }
        Ok(())
    }
    /// Validate inputs and build the physical indexed graph without a connectivity search.
    fn check_inputs(
        &self,
        setup: &Setup,
        terrain: &Terrain,
        food: &FoodLedger,
    ) -> Checked<Vec<bool>> {
        if (self.width, self.height) != (setup.width, setup.height)
            || terrain.dimensions() != (self.width, self.height)
            || setup.workers.len() != self.workers as usize
        {
            return Err(error(
                "access.dimensions",
                "setup, terrain and observed population disagree",
            ));
        }
        terrain.check()?;
        let initial: BTreeSet<_> = setup.open.iter().copied().collect();
        let mask: BTreeSet<_> = setup.diggable.iter().copied().collect();
        if initial.len() != setup.open.len()
            || mask.len() != setup.diggable.len()
            || terrain.counts().initial_open != initial.len() as u32
        {
            return Err(error(
                "access.setup",
                "initial geometry identities disagree",
            ));
        }
        let mut open = Vec::with_capacity((self.width * self.height) as usize);
        for index in 0..(self.width * self.height) as usize {
            let pos = self.position(index);
            let current = terrain.is_open(pos)?;
            if (current && !terrain.was_excavated(pos)?) != initial.contains(&pos)
                || terrain.is_diggable(pos)? != (!current && mask.contains(&pos))
            {
                return Err(error(
                    "access.setup",
                    "terrain history or eligible mask differs from setup",
                ));
            }
            open.push(current);
        }
        let mut resources = setup.food.clone();
        resources.sort_by_key(|r| r.id);
        if resources.len() > 256
            || resources.len() != food.views().len()
            || resources.windows(2).any(|w| w[0].id == w[1].id)
        {
            return Err(error("access.food", "food identities disagree with setup"));
        }
        for (resource, view) in resources.iter().zip(food.views()) {
            if *resource != view.resource
                || open[self.index(resource.pos)?] == (view.state == FoodState::Hidden)
            {
                return Err(error(
                    "access.food",
                    "food identity, original cell or exposure disagrees with physical terrain",
                ));
            }
        }
        Ok(open)
    }
    /// A local proof of shortest distances: predecessor paths and unit edge bounds, no BFS.
    fn check_graph(&self, setup: &Setup) -> Checked<()> {
        let nest: BTreeSet<_> = setup.nest.iter().copied().collect();
        let initial: BTreeSet<_> = setup.open.iter().copied().collect();
        let mask: BTreeSet<_> = setup.diggable.iter().copied().collect();
        if nest.len() < 2 || nest.len() != setup.nest.len() {
            return Err(error("access.nest", "requires distinct nest sources"));
        }
        for &pos in &nest {
            let index = self.index(pos)?;
            if !initial.contains(&pos)
                || !self.observed_open[index]
                || self.distances[index] != Some(0)
            {
                return Err(error(
                    "access.nest",
                    "nest sources must be initially open at distance zero",
                ));
            }
        }
        let mut excavated = 0;
        for (index, (&open, &distance)) in
            self.observed_open.iter().zip(&self.distances).enumerate()
        {
            let pos = self.position(index);
            if initial.contains(&pos) && !open
                || open && !initial.contains(&pos) && !mask.contains(&pos)
            {
                return Err(error(
                    "access.graph",
                    "observed graph contains an illegal revision",
                ));
            }
            excavated += u32::from(open && !initial.contains(&pos));
            if !open {
                if distance.is_some() {
                    return Err(error("access.distances", "closed cells require None"));
                }
                continue;
            }
            let adjacent = neighbors(pos, self.width, self.height);
            match distance {
                Some(0) if !nest.contains(&pos) => {
                    return Err(error(
                        "access.distances",
                        "only nest sources may have zero distance",
                    ))
                }
                Some(d)
                    if d > 0
                        && (d >= self.width * self.height
                            || !adjacent.iter().any(|p| {
                                self.distances[(p.y * self.width + p.x) as usize] == Some(d - 1)
                            })) =>
                {
                    return Err(error(
                        "access.distances",
                        "reachable non-nest cell needs an open predecessor one smaller",
                    ));
                }
                _ => {}
            }
            for next in adjacent {
                let j = (next.y * self.width + next.x) as usize;
                if !self.observed_open[j] {
                    continue;
                }
                match (distance, self.distances[j]) {
                    (Some(a), Some(b)) if a.abs_diff(b) <= 1 => {}
                    (None, None) => {}
                    _ => return Err(error(
                        "access.distances",
                        "adjacent open distances must differ by at most one and share reachability",
                    )),
                }
            }
        }
        if excavated != self.seen_digs {
            return Err(error(
                "access.seen_digs",
                "observed graph must contain exactly the observed excavations",
            ));
        }
        Ok(())
    }
    fn check_records(&self, setup: &Setup, food: &FoodLedger) -> Checked<()> {
        if self.records.len() != food.views().len() {
            return Err(error("access.records", "one stable record per food"));
        }
        let initial: BTreeSet<_> = setup.open.iter().copied().collect();
        for (record, view) in self.records.iter().zip(food.views()) {
            let pos = view.resource.pos;
            let index = self.index(pos)?;
            let distance = self.distance(pos)?;
            let exposed = self.observed_open[index];
            if record.id != view.resource.id
                || record.initially_exposed != initial.contains(&pos)
                || record.initially_accessible && !record.initially_exposed
                || record.accessible != distance.is_some()
                || record.distance != distance
                || record.initially_accessible && !record.accessible
                || record.first_exposure.is_some() != (!record.initially_exposed && exposed)
                || record.first_access.is_some()
                    != (!record.initially_accessible && record.accessible)
            {
                return Err(error(
                    format!("access.food[{}]", record.id),
                    "initial flags, event presence and cached current access disagree",
                ));
            }
            for event in [&record.first_exposure, &record.first_access]
                .into_iter()
                .flatten()
            {
                self.check_context(&event.context, event.context.excavated)?;
                if event.context.excavated == 0
                    || event.context.spoil_disposed >= event.context.excavated
                    || event.context.excavated > self.seen_digs
                    || event.context.opportunity > self.last_opportunity
                    || event.pos != pos
                    || event.context.food_delivered > food.inventory().delivered
                    || event.nest_distance >= self.width * self.height
                    || distance.is_none_or(|d| d > event.nest_distance)
                {
                    return Err(error(
                        format!("access.food[{}].event", record.id),
                        "milestone position, context bounds or frozen distance disagree",
                    ));
                }
            }
            if let (Some(exposure), Some(access)) = (&record.first_exposure, &record.first_access) {
                if exposure.context.opportunity > access.context.opportunity {
                    return Err(error(
                        "access.events",
                        "new access cannot precede food exposure",
                    ));
                }
            }
        }
        Ok(())
    }
    /// Exactly one multi-source BFS per constructor or successful candidate Dig update.
    fn bfs(&self, setup: &Setup, open: &[bool]) -> Checked<(Vec<Option<u32>>, AccessCompute)> {
        let mut distances = vec![None; (self.width * self.height) as usize];
        let mut queue = VecDeque::new();
        for &pos in &setup.nest {
            let index = self.index(pos)?;
            if !open[index] {
                return Err(error("access.nest", "source is closed"));
            }
            if distances[index].is_none() {
                distances[index] = Some(0);
                queue.push_back(pos);
            }
        }
        let mut compute = AccessCompute {
            calls: 1,
            visits: 0,
            peak_queue: queue.len() as u64,
        };
        while let Some(pos) = queue.pop_front() {
            compute.visits += 1;
            let next_distance =
                distances[self.index(pos)?].expect("queued cells have distance") + 1;
            for next in neighbors(pos, self.width, self.height) {
                let index = self.index(next)?;
                if open[index] && distances[index].is_none() {
                    distances[index] = Some(next_distance);
                    queue.push_back(next);
                    compute.peak_queue = compute.peak_queue.max(queue.len() as u64);
                }
            }
        }
        Ok((distances, compute))
    }
}
fn error(field: impl Into<String>, message: impl Into<String>) -> Vec<crate::config::FieldError> {
    vec![crate::config::FieldError::new(field, message)]
}
#[cfg(test)]
mod material_access {
    use super::super::tests::{pos, setup};
    use super::*;
    fn initial() -> (Setup, Terrain, FoodLedger, AccessObserver) {
        let s = setup();
        let t = Terrain::new(&s).unwrap();
        let f = FoodLedger::new(&s, &t).unwrap();
        let a = AccessObserver::new(&s, &t, &f).unwrap();
        (s, t, f, a)
    }
    #[test]
    fn new_dig_spoil_cannot_already_be_disposed() {
        let (s, mut t, mut f, mut a) = initial();
        t.dig(pos(3, 0)).unwrap();
        f.expose(pos(3, 0)).unwrap();
        let before = a.clone();
        let ctx = EventContext {
            tick: 0,
            opportunity: 1,
            worker: 0,
            excavated: 1,
            spoil_disposed: 1,
            food_delivered: 0,
        };
        assert!(a.after_dig(&s, &t, &f, &ctx).is_err());
        assert_eq!(a, before);
    }
    #[test]
    fn local_cache_check_rejects_all_inconsistent_shapes_and_distances() {
        let (s, t, f, a) = initial();
        for mode in 0..9 {
            let mut bad = a.clone();
            match mode {
                0 => bad.distances.pop().map(|_| ()).unwrap(),
                1 => bad.distances[3] = Some(2), // closed cell
                2 => bad.distances[0] = Some(1), // nest not zero
                3 => bad.distances[2] = Some(0), // non-nest zero
                4 => bad.distances[2] = Some(3), // missing predecessor / edge gap
                5 => bad.distances[2] = None,    // reachable None
                6 => bad.seen_digs = 1,
                7 => bad.compute.calls = 2,
                _ => bad.observed_open[2] = false,
            }
            assert!(bad.check(&s, &t, &f).is_err(), "mode {mode}");
        }
    }
    #[test]
    fn corrupted_prior_cache_cannot_be_repaired_by_successful_dig() {
        let (s, mut t, mut f, mut a) = initial();
        a.distances[2] = None;
        t.dig(pos(3, 0)).unwrap();
        f.expose(pos(3, 0)).unwrap();
        let before = a.clone();
        let ctx = EventContext {
            tick: 0,
            opportunity: 1,
            worker: 0,
            excavated: 1,
            spoil_disposed: 0,
            food_delivered: 0,
        };
        assert!(a.after_dig(&s, &t, &f, &ctx).is_err());
        assert_eq!(a, before);
    }
    #[test]
    fn invalid_observer_counter_bounds_preserve_state() {
        let (s, mut t, mut f, mut a) = initial();
        a.compute.visits = u64::MAX;
        t.dig(pos(3, 0)).unwrap();
        f.expose(pos(3, 0)).unwrap();
        let before = a.clone();
        let ctx = EventContext {
            tick: 0,
            opportunity: 1,
            worker: 0,
            excavated: 1,
            spoil_disposed: 0,
            food_delivered: 0,
        };
        assert!(a.after_dig(&s, &t, &f, &ctx).is_err());
        assert_eq!(a, before);
    }
    #[test]
    fn malformed_records_and_contexts_are_checked_locally() {
        let (s, mut t, mut f, mut a) = initial();
        t.dig(pos(3, 0)).unwrap();
        f.expose(pos(3, 0)).unwrap();
        let ctx = EventContext {
            tick: 10,
            opportunity: 11,
            worker: 0,
            excavated: 1,
            spoil_disposed: 0,
            food_delivered: 0,
        };
        a.after_dig(&s, &t, &f, &ctx).unwrap();
        for mode in 0..7 {
            let mut bad = a.clone();
            match mode {
                0 => bad.records[0].id = 9,
                1 => bad.records[0].initially_exposed = true,
                2 => bad.records[0].accessible = false,
                3 => bad.records[0].distance = Some(1),
                4 => bad.records[0].first_access.as_mut().unwrap().context.tick = 11,
                5 => bad.records[0].first_access.as_mut().unwrap().nest_distance = 1,
                _ => bad.records[0].first_exposure = None,
            };
            assert!(bad.check(&s, &t, &f).is_err(), "mode {mode}");
        }
    }
    #[test]
    fn multiple_or_disconnected_digs_are_rejected_without_observer_mutation() {
        let mut s = setup();
        s.diggable.push(pos(4, 0));
        let mut t = Terrain::new(&s).unwrap();
        let mut f = FoodLedger::new(&s, &t).unwrap();
        let mut a = AccessObserver::new(&s, &t, &f).unwrap();
        let before = a.clone();
        t.dig(pos(3, 0)).unwrap();
        t.dig(pos(4, 0)).unwrap();
        f.expose(pos(3, 0)).unwrap();
        let ctx = EventContext {
            tick: 0,
            opportunity: 1,
            worker: 0,
            excavated: 2,
            spoil_disposed: 0,
            food_delivered: 0,
        };
        assert!(a.after_dig(&s, &t, &f, &ctx).is_err());
        assert_eq!(a, before);
        let mut s = setup();
        s.diggable = vec![pos(4, 2)];
        let mut t = Terrain::new(&s).unwrap();
        let f = FoodLedger::new(&s, &t).unwrap();
        let mut a = AccessObserver::new(&s, &t, &f).unwrap();
        let before = a.clone();
        t.dig(pos(4, 2)).unwrap();
        let ctx = EventContext {
            excavated: 1,
            ..ctx
        };
        assert!(a.after_dig(&s, &t, &f, &ctx).is_err());
        assert_eq!(a, before);
    }
}
