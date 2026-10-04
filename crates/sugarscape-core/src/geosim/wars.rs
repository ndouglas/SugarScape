//! Generation-aware clusters and a complete census independent of legacy collection.
use super::config::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct StateId {
    pub capital_cell: usize,
    pub sovereignty_generation: u64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Participant {
    pub state: StateId,
    pub last_fighting_period: u64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct War {
    pub id: u64,
    pub parents: Vec<u64>,
    pub start_period: u64,
    pub end_period: Option<u64>,
    pub last_active_period: u64,
    pub active_periods: u64,
    pub elapsed_periods: u64,
    pub raw_severity: f64,
    pub exported_severity: f64,
    pub participants: Vec<Participant>,
    pub end_cause: Option<String>,
    pub java_saturated: bool,
    pub java_subunit_zero: bool,
    pub fighting_periods: BTreeSet<u64>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Merge {
    pub period: u64,
    pub survivor: u64,
    pub absorbed: u64,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WarTracker {
    pub active: BTreeMap<u64, War>,
    pub completed: Vec<War>,
    pub legacy_visible: Vec<War>,
    pub backlog: VecDeque<War>,
    pub merges: Vec<Merge>,
    pub next_id: u64,
}
pub fn java_int100(severity: f64) -> i32 {
    (severity * 100.0) as i32
}
impl WarTracker {
    fn alive(p: &Participant, period: u64, sov: &BTreeSet<StateId>, c: &GeosimConfig) -> bool {
        period.saturating_sub(p.last_fighting_period) <= c.war_shadow
            && (c.retired_participants == RetiredParticipants::RetainShadow
                || sov.contains(&p.state))
    }
    fn touching(
        &self,
        state: StateId,
        period: u64,
        sov: &BTreeSet<StateId>,
        c: &GeosimConfig,
    ) -> Vec<u64> {
        self.active
            .iter()
            .filter(|(_, w)| {
                w.participants
                    .iter()
                    .any(|p| p.state == state && Self::alive(p, period, sov, c))
            })
            .map(|(&id, _)| id)
            .collect()
    }
    fn join(&mut self, ids: &BTreeSet<u64>, period: u64) -> u64 {
        let survivor = *ids.first().expect("nonempty merge");
        for &id in ids.iter().skip(1) {
            let other = self.active.remove(&id).expect("merge member");
            let w = self.active.get_mut(&survivor).expect("merge survivor");
            w.raw_severity += other.raw_severity;
            w.start_period = w.start_period.min(other.start_period);
            w.last_active_period = w.last_active_period.max(other.last_active_period);
            w.parents.push(id);
            w.parents.extend(other.parents);
            w.parents.sort_unstable();
            w.parents.dedup();
            w.fighting_periods.extend(other.fighting_periods);
            for p in other.participants {
                if let Some(old) = w.participants.iter_mut().find(|q| q.state == p.state) {
                    old.last_fighting_period = old.last_fighting_period.max(p.last_fighting_period);
                } else {
                    w.participants.push(p);
                }
            }
            w.participants.sort_by_key(|p| p.state);
            self.merges.push(Merge {
                period,
                survivor,
                absorbed: id,
            });
        }
        survivor
    }
    fn refresh(w: &mut War, period: u64, c: &GeosimConfig) {
        w.elapsed_periods = period - w.start_period + 1;
        w.active_periods = w.fighting_periods.len() as u64;
        w.exported_severity = if c.severity_export == SeverityExport::JavaInt100 {
            f64::from(java_int100(w.raw_severity))
        } else {
            w.raw_severity
        };
        w.java_saturated = w.raw_severity * 100.0 > f64::from(i32::MAX);
        w.java_subunit_zero = w.raw_severity > 0.0 && java_int100(w.raw_severity) == 0;
    }
    pub fn advance(
        &mut self,
        period: u64,
        edges: &[(StateId, StateId, f64)],
        adjacent: &[(StateId, StateId)],
        sovereigns: &BTreeSet<StateId>,
        config: &GeosimConfig,
    ) {
        for &(a, b, damage) in edges {
            let mut ids: BTreeSet<u64> = self
                .touching(a, period, sovereigns, config)
                .into_iter()
                .chain(self.touching(b, period, sovereigns, config))
                .collect();
            if ids.is_empty() {
                let id = self.next_id;
                self.next_id += 1;
                self.active.insert(
                    id,
                    War {
                        id,
                        parents: Vec::new(),
                        start_period: period,
                        end_period: None,
                        last_active_period: period,
                        active_periods: 0,
                        elapsed_periods: 1,
                        raw_severity: 0.0,
                        exported_severity: 0.0,
                        participants: Vec::new(),
                        end_cause: None,
                        java_saturated: false,
                        java_subunit_zero: false,
                        fighting_periods: BTreeSet::new(),
                    },
                );
                ids.insert(id);
            }
            let id = self.join(&ids, period);
            let w = self.active.get_mut(&id).unwrap();
            w.raw_severity += damage;
            w.last_active_period = period;
            w.fighting_periods.insert(period);
            for state in [a, b] {
                if let Some(p) = w.participants.iter_mut().find(|p| p.state == state) {
                    p.last_fighting_period = period;
                } else {
                    w.participants.push(Participant {
                        state,
                        last_fighting_period: period,
                    });
                }
            }
            w.participants.sort_by_key(|p| p.state);
        }
        if config.cluster_linkage == ClusterLinkage::AdjacentActiveStates {
            for &(a, b) in adjacent {
                let ids: BTreeSet<_> = self
                    .touching(a, period, sovereigns, config)
                    .into_iter()
                    .chain(self.touching(b, period, sovereigns, config))
                    .collect();
                if ids.len() > 1 {
                    self.join(&ids, period);
                }
            }
        }
        for w in self.active.values_mut() {
            Self::refresh(w, period, config);
        }
        let done: Vec<_> = self
            .active
            .iter()
            .filter(|(_, w)| {
                !w.participants
                    .iter()
                    .any(|p| Self::alive(p, period, sovereigns, config))
            })
            .map(|(&id, _)| id)
            .collect();
        for id in done {
            let mut w = self.active.remove(&id).unwrap();
            w.end_period = Some(period);
            w.end_cause = Some("shadow_expired_or_participants_retired".into());
            if w.raw_severity > 0.0 {
                self.backlog.push_back(w.clone());
            }
            self.completed.push(w);
        }
        match config.completed_export {
            CompletedExport::AllCompleted => {
                while let Some(w) = self.backlog.pop_front() {
                    self.legacy_visible.push(w);
                }
            }
            CompletedExport::OnePerPeriod => {
                if let Some(w) = self.backlog.pop_front() {
                    self.legacy_visible.push(w);
                }
            }
        }
    }
    pub fn censored(&self, period: u64) -> Vec<War> {
        self.active
            .values()
            .cloned()
            .map(|mut w| {
                w.elapsed_periods = period - w.start_period + 1;
                w.end_cause = Some("horizon_censored".into());
                w
            })
            .collect()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn s(c: usize, g: u64) -> StateId {
        StateId {
            capital_cell: c,
            sovereignty_generation: g,
        }
    }
    #[test]
    fn merging_keeps_damage_once_and_never_splits() {
        let mut t = WarTracker::default();
        let c = GeosimConfig {
            war_shadow: 2,
            ..Default::default()
        };
        let sov = [s(0, 0), s(1, 0), s(2, 0), s(3, 0)].into_iter().collect();
        t.advance(
            1,
            &[(s(0, 0), s(1, 0), 2.0), (s(2, 0), s(3, 0), 3.0)],
            &[],
            &sov,
            &c,
        );
        t.advance(2, &[(s(1, 0), s(2, 0), 7.0)], &[], &sov, &c);
        assert_eq!(t.active.len(), 1);
        assert_eq!(t.active[&0].raw_severity, 12.0);
        assert_eq!(t.active[&0].active_periods, 2);
        assert_eq!(t.merges.len(), 1);
        t.advance(
            3,
            &[(s(0, 0), s(1, 0), 1.0), (s(2, 0), s(3, 0), 1.0)],
            &[],
            &sov,
            &c,
        );
        assert_eq!(t.active.len(), 1);
        assert_eq!(t.active[&0].raw_severity, 14.0);
    }
    #[test]
    fn shadow_expiry_and_reemergence_use_distinct_generations() {
        let mut t = WarTracker::default();
        let c = GeosimConfig {
            war_shadow: 2,
            ..Default::default()
        };
        let sov = [s(0, 1), s(1, 0)].into_iter().collect();
        t.advance(1, &[(s(0, 0), s(1, 0), 2.0)], &[], &sov, &c);
        t.advance(3, &[], &[], &sov, &c);
        assert_eq!(t.active.len(), 1);
        t.advance(4, &[], &[], &sov, &c);
        assert_eq!(t.completed.len(), 1);
        assert_eq!(t.completed[0].active_periods, 1);
        assert_eq!(t.completed[0].elapsed_periods, 4);
        assert_eq!(t.completed[0].participants[0].state, s(0, 0));
    }
    #[test]
    fn completed_backlog_is_not_censored_and_subunit_exports_survive() {
        let mut t = WarTracker::default();
        let c = GeosimConfig {
            war_shadow: 0,
            completed_export: CompletedExport::OnePerPeriod,
            severity_export: SeverityExport::JavaInt100,
            ..Default::default()
        };
        let sov = [s(0, 0), s(1, 0), s(2, 0), s(3, 0)].into_iter().collect();
        t.advance(
            1,
            &[(s(0, 0), s(1, 0), 0.001), (s(2, 0), s(3, 0), 3.0)],
            &[],
            &sov,
            &c,
        );
        t.advance(2, &[], &[], &sov, &c);
        assert_eq!(t.completed.len(), 2);
        assert_eq!(t.legacy_visible.len(), 1);
        assert_eq!(t.backlog.len(), 1);
        assert!(t.censored(2).is_empty());
        assert_eq!(t.completed[0].exported_severity, 0.0);
        assert!(t.completed[0].java_subunit_zero);
    }
    #[test]
    fn java_narrowing_saturates_instead_of_wrapping() {
        assert_eq!(java_int100(0.019), 1);
        assert_eq!(java_int100(1e100), i32::MAX);
        assert_eq!(java_int100(f64::NAN), 0);
    }
}
#[cfg(test)]
mod reading_tests {
    use super::*;
    fn s(capital_cell: usize) -> StateId {
        StateId {
            capital_cell,
            sovereignty_generation: 0,
        }
    }
    #[test]
    fn zero_severity_finishes_full_census_without_positive_fifo_entry() {
        let mut t = WarTracker::default();
        let c = GeosimConfig {
            war_shadow: 0,
            completed_export: CompletedExport::OnePerPeriod,
            ..Default::default()
        };
        let sov = [s(0), s(1)].into_iter().collect();
        t.advance(1, &[(s(0), s(1), 0.0)], &[], &sov, &c);
        t.advance(2, &[], &[], &sov, &c);
        assert_eq!(t.completed.len(), 1);
        assert!(t.legacy_visible.is_empty());
        assert!(t.backlog.is_empty());
    }
    #[test]
    fn adjacency_control_merges_separate_conflicts_and_retired_drop_completes() {
        let mut t = WarTracker::default();
        let mut c = GeosimConfig {
            cluster_linkage: ClusterLinkage::AdjacentActiveStates,
            ..Default::default()
        };
        let sov = [s(0), s(1), s(2), s(3)].into_iter().collect();
        t.advance(
            1,
            &[(s(0), s(1), 1.0), (s(2), s(3), 2.0)],
            &[(s(1), s(2))],
            &sov,
            &c,
        );
        assert_eq!(t.active.len(), 1);
        assert_eq!(t.active[&0].raw_severity, 3.0);
        c.retired_participants = RetiredParticipants::DropImmediately;
        t.advance(2, &[], &[], &BTreeSet::new(), &c);
        assert_eq!(t.completed.len(), 1);
    }
}
