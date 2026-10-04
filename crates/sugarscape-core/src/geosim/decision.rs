//! Double-buffered source decisions with explicit campaign and alert timing.
use super::*;
impl GeosimWorld {
    pub(crate) fn allocate(&mut self) -> Result<(), String> {
        let mut next = std::collections::BTreeMap::new();
        for (&id, s) in &self.states {
            let fronts: Vec<_> = self
                .relations
                .get(&id)
                .into_iter()
                .flatten()
                .map(|&k| (k, &self.fronts[&k], usize::from(k[1] == id)))
                .collect();
            let n = fronts.len();
            if n == 0 {
                continue;
            }
            let capacity = s.capacity.ok_or("unavailable resource capacity")?;
            let enemy: f64 = fronts
                .iter()
                .filter(|(_, f, _)| {
                    self.config.enemy_total == EnemyTotal::AllFronts
                        || f.previous.iter().any(|a| *a)
                })
                .map(|(_, f, side)| f.old_commitments[1 - side])
                .sum();
            for (key, f, side) in fronts {
                let v = super::resources::commitment(
                    capacity,
                    self.config.mobile_share,
                    n,
                    f.old_commitments[1 - side],
                    enemy,
                    f.previous.iter().any(|a| *a),
                );
                if !v.is_finite() {
                    return Err(format!(
                        "period {} front {:?}: nonfinite allocation",
                        self.period, key
                    ));
                }
                next.entry(key).or_insert([0.0; 2])[side] = v;
            }
        }
        for (key, c) in next {
            self.fronts.get_mut(&key).unwrap().commitments = c;
        }
        Ok(())
    }
    pub(crate) fn decide(&mut self) -> Result<(), String> {
        let mut fighting: std::collections::BTreeSet<_> = self
            .fronts
            .values()
            .filter(|f| f.previous.iter().any(|a| *a))
            .flat_map(|f| f.states)
            .collect();
        fighting.extend(self.prior_fighting.iter().copied());
        for f in self.fronts.values_mut() {
            f.actions = [f.previous.iter().any(|a| *a); 2];
        }
        let mut actors: Vec<_> = self.states.keys().copied().collect();
        super::world::shuffle(&mut actors, &mut self.rng);
        let mut new_paths = std::collections::BTreeMap::<[StateId; 2], StateId>::new();
        for id in actors {
            let keys = self.relations.get(&id).cloned().unwrap_or_default();
            if keys.is_empty() {
                continue;
            }
            let neighbors: Vec<_> = keys
                .iter()
                .map(|k| if k[0] == id { k[1] } else { k[0] })
                .collect();
            let neighborhood_active =
                fighting.contains(&id) || neighbors.iter().any(|n| fighting.contains(n));
            let s = self.states.get_mut(&id).unwrap();
            if !self.config.context_activation {
                s.alert = false;
            } else if neighborhood_active {
                s.alert = true;
            } else if s.alert
                && super::world::chance(self.config.deactivation_probability, &mut self.rng)
            {
                s.alert = false;
            }
            if s.campaign.is_some_and(|t| !neighbors.contains(&t)) {
                s.campaign = None;
            }
            if s.campaign.is_some()
                && self.config.campaign_drop_timing == CampaignDropTiming::EachDecision
                && super::world::chance(self.config.campaign_drop_probability, &mut self.rng)
            {
                s.campaign = None;
            }
            let no_action = !keys
                .iter()
                .any(|k| self.fronts[k].actions[usize::from(k[1] == id)]);
            let bypass = s.alert || s.campaign.is_some();
            let eligible = match self.config.initiation_guard {
                InitiationGuard::LiteralPrecedence => bypass || no_action,
                InitiationGuard::GlobalNoAction => no_action,
            };
            if !eligible {
                continue;
            }
            let contemplated =
                bypass || super::world::chance(self.config.attack_probability, &mut self.rng);
            if !contemplated {
                continue;
            }
            let target = match self.states[&id].campaign {
                Some(t) => t,
                None => super::world::pick(&neighbors, &mut self.rng).unwrap(),
            };
            let key = if id < target {
                [id, target]
            } else {
                [target, id]
            };
            let side = usize::from(key[1] == id);
            let path = self
                .select_path(key, side)
                .ok_or("neighbor front has no valid border path")?;
            let commits = self.fronts[&key].commitments;
            let curve_target = self.attack_defender_projection(id, target, path[1 - side]);
            let attack = commits[side] * self.projection(id, path[side]);
            let defense = commits[1 - side] * curve_target;
            let p = super::combat::contest_probability(
                attack,
                defense,
                self.config.superiority_threshold,
                self.config.superiority_exponent,
                self.config.numerical_policy,
            )
            .map_err(|e| format!("period {} initiation {:?}: {}", self.period, key, e))?;
            if super::world::chance(p, &mut self.rng) {
                let f = self.fronts.get_mut(&key).unwrap();
                f.actions[side] = true;
                if new_paths.contains_key(&key) {
                    self.ledger.path_collisions += 1;
                }
                if new_paths.get(&key).is_none_or(|old| id < *old) {
                    f.path = Some(path);
                    f.initiator = Some(side);
                    new_paths.insert(key, id);
                }
                self.states.get_mut(&id).unwrap().campaign = Some(target);
                self.ledger.attacks += 1;
                self.log_event("attack", vec![id, target], path.to_vec());
            }
        }
        let missing: Vec<_> = self
            .fronts
            .iter()
            .filter(|(_, f)| f.actions.iter().any(|a| *a) && f.path.is_none())
            .map(|(&key, f)| (key, f.initiator.unwrap_or(0)))
            .collect();
        for (key, side) in missing {
            let path = self
                .select_path(key, side)
                .ok_or("fighting relation has no border")?;
            let f = self.fronts.get_mut(&key).unwrap();
            f.path = Some(path);
            f.initiator = Some(side);
        }
        Ok(())
    }
    pub(crate) fn attack_defender_projection(
        &self,
        initiator: StateId,
        target: StateId,
        cell: usize,
    ) -> f64 {
        if self.config.attack_projection == AttackProjection::InitiatorCurve {
            super::resources::distance_curve(
                super::territory::distance(&self.config, target.capital_cell, cell),
                self.config.distance_offset,
                self.states[&initiator].threshold,
                self.config.distance_exponent,
                self.config.distance_formula == DistanceFormula::PrintedIncreasing,
            )
        } else {
            self.projection(target, cell)
        }
    }
    pub(crate) fn select_path(
        &mut self,
        states: [StateId; 2],
        attacker: usize,
    ) -> Option<[usize; 2]> {
        let first = if self.config.path_sampling == PathSampling::TargetFirst {
            1 - attacker
        } else {
            attacker
        };
        let second = 1 - first;
        let border: Vec<_> = self.members[&states[first]]
            .iter()
            .copied()
            .filter(|&id| {
                super::territory::adjacent(&self.config, id)
                    .iter()
                    .any(|&j| self.cells[j].owner == states[second])
            })
            .collect();
        let a = super::world::pick(&border, &mut self.rng)?;
        let neighbors: Vec<_> = super::territory::adjacent(&self.config, a)
            .into_iter()
            .filter(|&id| self.cells[id].owner == states[second])
            .collect();
        let b = super::world::pick(&neighbors, &mut self.rng)?;
        let mut path = [0; 2];
        path[first] = a;
        path[second] = b;
        Some(path)
    }
}
#[cfg(test)]
mod tests {
    use super::super::tests::prescribed;
    use super::*;
    use rand::RngCore;
    #[test]
    fn singleton_selection_does_not_consume_a_genuine_random_draw() {
        let mut a = crate::rng::seeded(9);
        let mut b = a.clone();
        assert_eq!(super::super::world::pick(&[42], &mut a), Some(42));
        assert_eq!(a.next_u64(), b.next_u64());
    }
    #[test]
    fn grim_trigger_and_neighbor_alert_use_old_action_buffer() {
        let mut w = prescribed(&[0, 0, 2, 2], 2);
        let key = *w.fronts.keys().next().unwrap();
        w.fronts.get_mut(&key).unwrap().previous = [true, false];
        w.allocate().unwrap();
        w.decide().unwrap();
        assert_eq!(w.fronts[&key].actions, [true, true]);
        assert!(w.states.values().all(|s| s.alert));
    }
    #[test]
    fn context_disabled_retains_spontaneous_contemplation() {
        let mut w = prescribed(&[0, 0, 2, 2], 2);
        w.config.context_activation = false;
        w.config.attack_probability = 1.0;
        w.config.superiority_threshold = 1e-30;
        w.allocate().unwrap();
        w.decide().unwrap();
        assert_eq!(w.ledger.attacks, 2);
        assert_eq!(w.ledger.path_collisions, 1);
        assert!(w.fronts.values().all(|f| f.actions == [true, true]));
        assert!(w.states.values().all(|s| !s.alert));
    }
    #[test]
    fn path_sampler_only_selects_adjacent_owned_cells() {
        let mut w = prescribed(&[0, 0, 2, 2], 2);
        let key = *w.fronts.keys().next().unwrap();
        for mode in [PathSampling::TargetFirst, PathSampling::AttackerFirst] {
            w.config.path_sampling = mode;
            let [a, b] = w.select_path(key, 0).unwrap();
            assert_eq!(w.cells[a].owner, key[0]);
            assert_eq!(w.cells[b].owner, key[1]);
            assert!(super::super::territory::adjacent(&w.config, a).contains(&b));
        }
    }
    #[test]
    fn hybrid_allocation_uses_full_mobile_for_conditional_quiet_fronts() {
        let mut w = prescribed(&[0, 0, 2, 2], 2);
        w.allocate().unwrap();
        assert_eq!(w.fronts.values().next().unwrap().commitments, [20.0, 20.0]);
    }
}
#[cfg(test)]
mod reading_tests {
    use super::super::tests::prescribed;
    use super::*;
    #[test]
    fn all_front_denominator_differs_from_active_front_example() {
        let mut w = prescribed(&[0, 0, 2, 0, 0, 5], 3);
        let own = w.cells[0].owner;
        w.states.get_mut(&own).unwrap().capacity = Some(100.0);
        let keys = w.relations[&own].clone();
        w.fronts.get_mut(&keys[0]).unwrap().previous = [true, true];
        w.fronts.get_mut(&keys[0]).unwrap().old_commitments[1] = 10.0;
        w.fronts.get_mut(&keys[1]).unwrap().old_commitments[1] = 30.0;
        w.allocate().unwrap();
        assert_eq!(w.fronts[&keys[0]].commitments[0], 75.0);
        assert_eq!(w.fronts[&keys[1]].commitments[0], 62.5);
        w.config.enemy_total = EnemyTotal::AllFronts;
        w.allocate().unwrap();
        assert_eq!(w.fronts[&keys[0]].commitments[0], 37.5);
    }
    #[test]
    fn quiet_deactivation_and_campaign_drop_timing_are_distinct() {
        let mut w = prescribed(&[0, 0, 2, 2], 2);
        let own = w.cells[0].owner;
        let other = w.cells[2].owner;
        w.config.deactivation_probability = 1.0;
        w.config.campaign_drop_probability = 1.0;
        w.config.superiority_threshold = 1e100;
        w.states.get_mut(&own).unwrap().alert = true;
        w.states.get_mut(&own).unwrap().campaign = Some(other);
        w.allocate().unwrap();
        w.decide().unwrap();
        assert!(!w.states[&own].alert);
        assert_eq!(w.states[&own].campaign, None);
        w.config.campaign_drop_timing = CampaignDropTiming::AfterBattle;
        w.states.get_mut(&own).unwrap().campaign = Some(other);
        w.decide().unwrap();
        assert_eq!(w.states[&own].campaign, Some(other));
    }
}
#[cfg(test)]
mod completed_battle_memory_tests {
    use super::super::tests::prescribed;
    use super::*;
    #[test]
    fn completed_battle_still_alerts_neighborhood_next_period() {
        let mut w = prescribed(&[0, 0, 2, 2], 2);
        w.config.attack_probability = 1.0;
        w.config.superiority_threshold = 1e-30;
        w.config.victory_threshold = 1e-30;
        w.config.defender_threshold = DefenderThreshold::SameThreshold;
        w.config.deactivation_probability = 0.0;
        w.run(1);
        assert!(w.fronts.values().all(|f| f.previous == [false, false]));
        assert!(w.states.values().all(|s| !s.alert));
        w.run(1);
        assert!(w.states.values().all(|s| s.alert));
    }
}
#[cfg(test)]
mod portable_index_tests {
    use super::super::world::pick;
    use rand::Rng;
    #[test]
    fn random_selection_uses_fixed_u32_draws_for_native_wasm_parity() {
        let values: Vec<u32> = (0..10000).collect();
        let mut actual = crate::rng::seeded(123);
        let mut reference = actual.clone();
        for _ in 0..5 {
            let expected = reference.gen_range(0_u32..10000_u32);
            assert_eq!(pick(&values, &mut actual), Some(expected));
        }
    }
}

#[cfg(test)]
mod projection_reading_tests {
    use super::super::tests::prescribed;
    use super::*;
    #[test]
    fn initiator_curve_changes_threshold_but_preserves_defender_own_capital_distance() {
        let mut w = prescribed(&[0, 0, 0, 7, 0, 0, 7, 7], 4);
        let initiator = w.cells[0].owner;
        let defender = w.cells[7].owner;
        w.states.get_mut(&initiator).unwrap().threshold = 1.0;
        w.states.get_mut(&defender).unwrap().threshold = 10.0;
        let own_curve = w.projection(defender, 3);
        assert_eq!(
            w.attack_defender_projection(initiator, defender, 3),
            own_curve
        );
        w.config.attack_projection = AttackProjection::InitiatorCurve;
        let expected = super::super::resources::distance_curve(1.0, 0.1, 1.0, 3.0, false);
        assert!((w.attack_defender_projection(initiator, defender, 3) - expected).abs() < 1e-12);
        assert!((expected - w.projection(initiator, 3)).abs() > 0.4);
    }
}

#[cfg(test)]
mod guard_reading_tests {
    use super::super::tests::prescribed;
    use super::*;
    #[test]
    fn previous_defection_blocks_global_initiation_but_literal_alert_bypasses_it() {
        for (guard, expected_attacks) in [
            (InitiationGuard::GlobalNoAction, 0),
            (InitiationGuard::LiteralPrecedence, 2),
        ] {
            let mut w = prescribed(&[0, 0, 2, 2], 2);
            w.config.initiation_guard = guard;
            w.config.superiority_threshold = 1e-30;
            let key = *w.fronts.keys().next().unwrap();
            w.fronts.get_mut(&key).unwrap().previous = [true, false];
            w.allocate().unwrap();
            w.decide().unwrap();
            assert_eq!(w.ledger.attacks, expected_attacks);
            assert!(w.fronts.values().all(|f| f.actions == [true, true]));
        }
    }
}
