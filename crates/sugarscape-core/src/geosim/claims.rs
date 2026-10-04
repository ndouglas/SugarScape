//! Snapshot structural claims, contiguity and sovereign identity retirement.
use super::*;
#[derive(Clone, Debug)]
pub(crate) struct Claim {
    pub states: [StateId; 2],
    pub path: [usize; 2],
    pub attacker: usize,
}
impl GeosimWorld {
    fn prospective_owner_changes(
        &self,
        loser: StateId,
        target: usize,
        before_loser: &[usize],
    ) -> std::collections::BTreeSet<usize> {
        let mut changed = std::collections::BTreeSet::from([target]);
        if target == loser.capital_cell {
            if before_loser.len() > 1 {
                changed.extend(before_loser.iter().copied().filter(|&cell| {
                    cell != target || self.config.capital_capture == CapitalCapture::CollapseOnly
                }));
            }
        } else {
            let mut reached = std::collections::BTreeSet::new();
            let mut pending = vec![loser.capital_cell];
            while let Some(cell) = pending.pop() {
                if !reached.insert(cell) {
                    continue;
                }
                for next in super::territory::adjacent(&self.config, cell) {
                    if next != target && self.cells[next].owner == loser && !reached.contains(&next)
                    {
                        pending.push(next);
                    }
                }
            }
            changed.extend(
                before_loser
                    .iter()
                    .copied()
                    .filter(|&cell| cell != target && !reached.contains(&cell)),
            );
        }
        changed
    }

    fn release(&mut self, cell: usize) -> StateId {
        self.cells[cell].next_generation += 1;
        let id = StateId {
            capital_cell: cell,
            sovereignty_generation: self.cells[cell].next_generation,
        };
        let threshold =
            if self.config.technology_inheritance == TechnologyInheritance::RetainCellThreshold {
                self.cells[cell].last_threshold
            } else {
                self.config.distance_threshold
            };
        self.cells[cell].owner = id;
        self.states.insert(
            id,
            State {
                id,
                capacity: Some(1.0),
                threshold,
                alert: false,
                campaign: None,
                previous_damage: 0.0,
                newly_independent: false,
                extracted_yield: 1.0,
                recurrence_residual: 0.0,
            },
        );
        self.ledger.reemergence_capacity += 1.0;
        id
    }
    pub(crate) fn apply_claims(&mut self, mut claims: Vec<Claim>) -> Result<(), String> {
        super::world::shuffle(&mut claims, &mut self.rng);
        let mut locked = std::collections::BTreeSet::new();
        for claim in claims {
            let winner = claim.states[claim.attacker];
            let loser = claim.states[1 - claim.attacker];
            let agent = claim.path[claim.attacker];
            let target = claim.path[1 - claim.attacker];
            if !self.states.contains_key(&winner)
                || !self.states.contains_key(&loser)
                || self.cells[agent].owner != winner
                || self.cells[target].owner != loser
                || !super::territory::adjacent(&self.config, agent).contains(&target)
            {
                self.ledger.stale_claims += 1;
                self.log_event("stale_claim", claim.states.to_vec(), claim.path.to_vec());
                continue;
            }
            let before_winner = self.members[&winner].clone();
            let before_loser = self.members[&loser].clone();
            let prospective_changes = self.prospective_owner_changes(loser, target, &before_loser);
            if locked.contains(&agent)
                || prospective_changes.iter().any(|cell| locked.contains(cell))
            {
                self.ledger.locked_claims += 1;
                self.log_event("locked_claim", claim.states.to_vec(), claim.path.to_vec());
                continue;
            }
            let mut changed = vec![target, winner.capital_cell, loser.capital_cell];
            if target == loser.capital_cell {
                let old = self.states.remove(&loser).unwrap();
                self.cells[target].last_threshold = old.threshold;
                self.retired.push(loser);
                self.ledger.retirement_capacity += old.capacity.unwrap_or(0.0);
                if before_loser.len() > 1 {
                    self.ledger.collapses += 1;
                    for &cell in &before_loser {
                        if cell != target
                            || self.config.capital_capture == CapitalCapture::CollapseOnly
                        {
                            self.release(cell);
                            changed.push(cell);
                        }
                    }
                }
                if before_loser.len() == 1
                    || self.config.capital_capture == CapitalCapture::CaptureAndFragment
                {
                    self.cells[target].owner = winner;
                }
            } else {
                self.cells[target].owner = winner;
                let mut reached = std::collections::BTreeSet::new();
                let mut pending = vec![loser.capital_cell];
                while let Some(cell) = pending.pop() {
                    if !reached.insert(cell) {
                        continue;
                    }
                    for next in super::territory::adjacent(&self.config, cell) {
                        if self.cells[next].owner == loser && !reached.contains(&next) {
                            pending.push(next);
                        }
                    }
                }
                for cell in before_loser
                    .iter()
                    .copied()
                    .filter(|&id| id != target && !reached.contains(&id))
                {
                    self.release(cell);
                    changed.push(cell);
                    self.ledger.disconnections += 1;
                }
            }
            self.ledger.conquests += 1;
            self.log_event("conquest", vec![winner, loser], changed.clone());
            if self.config.locking == Locking::AffectedStates {
                locked.extend(before_winner);
                locked.extend(before_loser);
            } else {
                locked.extend(changed);
            }
            self.rebuild();
            let states: Vec<_> = self.states.keys().copied().collect();
            for id in states {
                let neighbors: std::collections::BTreeSet<_> = self
                    .fronts
                    .keys()
                    .filter(|k| k.contains(&id))
                    .flat_map(|k| k.iter().copied().filter(|n| *n != id))
                    .collect();
                if self.states[&id]
                    .campaign
                    .is_some_and(|t| !neighbors.contains(&t))
                {
                    self.states.get_mut(&id).unwrap().campaign = None;
                }
            }
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::super::tests::prescribed;
    use super::*;
    #[test]
    fn capital_capture_annexes_center_and_releases_new_generations() {
        let mut w = prescribed(&[0, 0, 2, 2], 2);
        let states = [w.cells[0].owner, w.cells[2].owner];
        w.apply_claims(vec![Claim {
            states,
            path: [0, 2],
            attacker: 0,
        }])
        .unwrap();
        assert_eq!(w.cells[2].owner, states[0]);
        assert_eq!(
            w.cells[3].owner,
            StateId {
                capital_cell: 3,
                sovereignty_generation: 1
            }
        );
        assert!(w.retired.contains(&states[1]));
        assert_eq!(w.states.len(), 2);
    }
    #[test]
    fn collapse_only_releases_center_without_annexing_it() {
        let mut w = prescribed(&[0, 0, 2, 2], 2);
        w.config.capital_capture = CapitalCapture::CollapseOnly;
        let states = [w.cells[0].owner, w.cells[2].owner];
        w.apply_claims(vec![Claim {
            states,
            path: [0, 2],
            attacker: 0,
        }])
        .unwrap();
        assert_eq!(
            w.cells[2].owner,
            StateId {
                capital_cell: 2,
                sovereignty_generation: 1
            }
        );
        assert_eq!(w.states.len(), 3);
    }
    #[test]
    fn province_loss_releases_disconnected_cells() {
        let mut w = prescribed(&[0, 0, 0, 3, 3, 3], 3);
        let states = [w.cells[0].owner, w.cells[3].owner];
        w.apply_claims(vec![Claim {
            states,
            path: [1, 4],
            attacker: 0,
        }])
        .unwrap();
        assert_eq!(w.cells[4].owner, states[0]);
        assert_eq!(
            w.cells[5].owner,
            StateId {
                capital_cell: 5,
                sovereignty_generation: 1
            }
        );
        assert_eq!(w.ledger.disconnections, 1);
    }
    #[test]
    fn stale_duplicate_capital_claim_never_resurrects_retired_state() {
        let mut w = prescribed(&[0, 0, 2, 2], 2);
        let states = [w.cells[0].owner, w.cells[2].owner];
        let claim = Claim {
            states,
            path: [0, 2],
            attacker: 0,
        };
        w.apply_claims(vec![claim.clone(), claim]).unwrap();
        assert_eq!(w.retired.iter().filter(|&&s| s == states[1]).count(), 1);
        assert_eq!(w.ledger.stale_claims, 1);
        assert!(!w.states.contains_key(&states[1]));
    }

    #[test]
    fn later_claim_cannot_release_a_province_locked_by_an_earlier_claim() {
        let mut w = prescribed(&[0, 0, 0, 4, 4, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7], 5);
        let a = w.cells[0].owner;
        let b = w.cells[4].owner;
        let c = w.cells[7].owner;
        let claims = vec![
            Claim {
                states: [a, b],
                path: [2, 3],
                attacker: 0,
            },
            Claim {
                states: [a, c],
                path: [2, 7],
                attacker: 1,
            },
        ];
        let seed = (0..1000)
            .find(|&seed| {
                let mut order = [0, 1];
                super::super::world::shuffle(&mut order, &mut crate::rng::seeded(seed));
                order == [0, 1]
            })
            .unwrap();
        w.rng = crate::rng::seeded(seed);

        w.apply_claims(claims).unwrap();

        assert_eq!(w.cells[3].owner, a);
        assert_eq!(w.ledger.locked_claims, 1);
        assert_eq!(w.ledger.conquests, 1);
    }
}
#[cfg(test)]
mod reading_tests {
    use super::super::tests::prescribed;
    use super::*;
    #[test]
    fn retained_technology_uses_cells_own_threshold_not_former_owner() {
        let mut w = prescribed(&[0, 0, 2, 2], 2);
        w.config.technology_inheritance = TechnologyInheritance::RetainCellThreshold;
        let states = [w.cells[0].owner, w.cells[2].owner];
        w.states.get_mut(&states[1]).unwrap().threshold = 9.0;
        w.cells[3].last_threshold = 4.0;
        w.apply_claims(vec![Claim {
            states,
            path: [0, 2],
            attacker: 0,
        }])
        .unwrap();
        assert_eq!(w.states[&w.cells[3].owner].threshold, 4.0);
    }
    #[test]
    fn affected_cells_allows_distinct_provinces_while_state_lock_blocks_second() {
        let base = prescribed(&[0, 0, 0, 3, 3, 3, 3, 3, 3], 3);
        let states = [base.cells[0].owner, base.cells[3].owner];
        let claims = vec![
            Claim {
                states,
                path: [1, 4],
                attacker: 0,
            },
            Claim {
                states,
                path: [2, 5],
                attacker: 0,
            },
        ];
        let mut cells = base.clone();
        cells.apply_claims(claims.clone()).unwrap();
        assert_eq!(cells.ledger.conquests, 2);
        let mut states_lock = base;
        states_lock.config.locking = Locking::AffectedStates;
        states_lock.apply_claims(claims).unwrap();
        assert_eq!(states_lock.ledger.conquests, 1);
        assert_eq!(states_lock.ledger.locked_claims, 1);
    }
}

#[cfg(test)]
mod unbounded_inspection_tests {
    use super::super::tests::prescribed;
    use super::*;
    use crate::model::Model;
    #[test]
    fn structural_event_is_inspectable_even_with_ui_trace_disabled() {
        let mut w = prescribed(&[0, 0, 2, 2], 2);
        w.config.event_log = false;
        let states = [w.cells[0].owner, w.cells[2].owner];
        w.apply_claims(vec![Claim {
            states,
            path: [0, 2],
            attacker: 0,
        }])
        .unwrap();
        let inspect: serde_json::Value =
            serde_json::from_str(&w.inspect_json(1, 1).unwrap()).unwrap();
        assert_eq!(inspect["last_structural_event"]["kind"], "conquest");
        assert_eq!(inspect["events"], serde_json::json!([]));
    }
}
