//! Source probability, damage and defender-priority resolution boundaries.
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Victory {
    Undecided,
    Attacker,
    Defender,
}
pub fn contest_probability(
    a: f64,
    b: f64,
    threshold: f64,
    exponent: u32,
    policy: NumericalPolicy,
) -> Result<f64, String> {
    if !a.is_finite() || !b.is_finite() || a < 0.0 || b < 0.0 {
        return Err(format!("invalid projected commitments {a}/{b}"));
    }
    if policy == NumericalPolicy::FloorZero {
        if a == 0.0 && b == 0.0 {
            return Ok(0.5);
        }
        if b == 0.0 {
            return Ok(1.0);
        }
        if a == 0.0 {
            return Ok(0.0);
        }
    }
    if a <= 0.0 || b <= 0.0 {
        return Err(format!("nonpositive projected commitments {a}/{b}"));
    }
    let ratio = a / b;
    if !ratio.is_finite() || ratio <= 0.0 {
        return Err(format!("invalid ratio {a}/{b}"));
    }
    let p = super::resources::probability(ratio, threshold, exponent);
    if !p.is_finite() || !(0.0..=1.0).contains(&p) {
        Err("invalid contest probability".into())
    } else {
        Ok(p)
    }
}
pub fn resolve_victory(prob: [f64; 2], draws: [f64; 2], mode: VictoryDraws) -> (Victory, bool) {
    if mode == VictoryDraws::IndependentDefenderPriority {
        let a = draws[0] < prob[0];
        let d = draws[1] < prob[1];
        (
            if d {
                Victory::Defender
            } else if a {
                Victory::Attacker
            } else {
                Victory::Undecided
            },
            a && d,
        )
    } else {
        (
            if draws[0] < prob[0] {
                Victory::Attacker
            } else if draws[0] < prob[0] + (1.0 - prob[0]) * prob[1] {
                Victory::Defender
            } else {
                Victory::Undecided
            },
            false,
        )
    }
}
pub fn losses(
    actions: [bool; 2],
    commitments: [f64; 2],
    projected: [f64; 2],
    fraction: f64,
    basis: DamageBasis,
) -> [f64; 2] {
    losses_with_incidence(
        actions,
        commitments,
        projected,
        fraction,
        basis,
        DamageIncidence::AttackedParty,
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn simultaneous_independent_successes_cancel_attacker_conquest() {
        assert_eq!(
            resolve_victory(
                [0.7, 0.6],
                [0.1, 0.1],
                VictoryDraws::IndependentDefenderPriority
            ),
            (Victory::Defender, true)
        );
    }
    #[test]
    fn exclusive_draw_can_leave_battle_undecided() {
        assert_eq!(
            resolve_victory([0.2, 0.3], [0.8, 0.0], VictoryDraws::Exclusive),
            (Victory::Undecided, false)
        );
        assert_eq!(
            resolve_victory([0.2, 0.3], [0.3, 0.0], VictoryDraws::Exclusive),
            (Victory::Defender, false)
        );
    }
    #[test]
    fn unilateral_attack_costs_victim_and_mutual_damage_is_projected() {
        assert_eq!(
            losses(
                [true, false],
                [20.0, 10.0],
                [10.0, 2.0],
                0.1,
                DamageBasis::OpponentProjected
            ),
            [0.0, 1.0]
        );
        assert_eq!(
            losses(
                [true, true],
                [20.0, 10.0],
                [10.0, 2.0],
                0.1,
                DamageBasis::OpponentProjected
            ),
            [0.2, 1.0]
        );
        assert_eq!(
            losses(
                [true, false],
                [20.0, 10.0],
                [10.0, 2.0],
                0.1,
                DamageBasis::OwnCommitment
            ),
            [0.0, 1.0]
        );
    }
    #[test]
    fn zero_ratios_are_explicit_and_invalid_ratios_fail() {
        assert!(
            contest_probability(0.0, 0.0, 3.0, 20, NumericalPolicy::RejectNonpositive).is_err()
        );
        assert_eq!(
            contest_probability(0.0, 0.0, 3.0, 20, NumericalPolicy::FloorZero).unwrap(),
            0.5
        );
        assert_eq!(
            contest_probability(1.0, 0.0, 3.0, 20, NumericalPolicy::FloorZero).unwrap(),
            1.0
        );
        assert!(
            contest_probability(f64::INFINITY, 1.0, 3.0, 20, NumericalPolicy::FloorZero).is_err()
        );
    }
}
type FightResolution = (Vec<super::claims::Claim>, Vec<(StateId, StateId, f64)>);
impl GeosimWorld {
    pub(crate) fn fight(&mut self) -> Result<FightResolution, String> {
        use rand::Rng;
        let mut claims = Vec::new();
        let mut edges = Vec::new();
        let keys: Vec<_> = self.fronts.keys().copied().collect();
        for key in keys {
            let f = self.fronts[&key].clone();
            if !f.actions.iter().any(|a| *a) {
                continue;
            }
            let path = f.path.ok_or("fighting front missing path")?;
            let projected = [
                f.commitments[0] * self.projection(key[0], path[0]),
                f.commitments[1] * self.projection(key[1], path[1]),
            ];
            let damage = losses_with_incidence(
                f.actions,
                f.commitments,
                projected,
                self.config.damage_fraction,
                self.config.damage_basis,
                self.config.damage_incidence,
            );
            if projected
                .iter()
                .chain(damage.iter())
                .any(|x| !x.is_finite() || *x < 0.0)
            {
                return Err(format!(
                    "period {} front {:?}: invalid damage/projection",
                    self.period, key
                ));
            }
            self.fronts.get_mut(&key).unwrap().last_damage = damage;
            for side in 0..2 {
                self.states.get_mut(&key[side]).unwrap().previous_damage += damage[side];
            }
            self.ledger.damage += damage.iter().sum::<f64>();
            self.ledger.fighting_front_periods += 1;
            if f.actions == [true, true] {
                self.ledger.mutual_front_periods += 1;
            }
            let severity = if self.config.severity_damage == SeverityDamage::AllDamagedFronts
                || f.actions == [true, true]
            {
                damage.iter().sum()
            } else {
                0.0
            };
            edges.push((key[0], key[1], severity));
            self.pending_fights.push((key[0], key[1], severity));
            if self.period >= self.counting_start() {
                self.ledger.measured_damage += severity;
            }
            let attacker = f.initiator.unwrap_or(usize::from(!f.actions[0]));
            let [p_a, p_d] = victory_probabilities(projected, attacker, &self.config)
                .map_err(|e| format!("period {} battle {:?}: {}", self.period, key, e))?;
            self.fronts
                .get_mut(&key)
                .unwrap()
                .last_victory_probabilities = [Some(p_a), Some(p_d)];
            let draws = [
                self.rng.gen::<f64>(),
                if self.config.victory_draws == VictoryDraws::IndependentDefenderPriority {
                    self.rng.gen::<f64>()
                } else {
                    0.0
                },
            ];
            let (win, both) = resolve_victory([p_a, p_d], draws, self.config.victory_draws);
            self.ledger.double_successes += u64::from(both);
            if win != Victory::Undecided {
                let current = self.fronts.get_mut(&key).unwrap();
                current.actions = [false; 2];
                current.initiator = None;
                self.log_event(
                    if win == Victory::Attacker {
                        "attacker_victory"
                    } else {
                        "defender_victory"
                    },
                    key.to_vec(),
                    path.to_vec(),
                );
                if win == Victory::Attacker {
                    claims.push(super::claims::Claim {
                        states: key,
                        path,
                        attacker,
                    });
                }
                if self.config.campaign_drop_timing == CampaignDropTiming::AfterBattle {
                    for id in key {
                        if self.states[&id].campaign.is_some()
                            && super::world::chance(
                                self.config.campaign_drop_probability,
                                &mut self.rng,
                            )
                        {
                            self.states.get_mut(&id).unwrap().campaign = None;
                        }
                    }
                }
            }
        }
        Ok((claims, edges))
    }
}
#[cfg(test)]
mod integrated_tests {
    use super::super::tests::prescribed;
    use super::*;
    #[test]
    fn unilateral_front_refreshes_war_with_named_severity_filter() {
        let mut w = prescribed(&[0, 0, 2, 2], 2);
        w.period = 1;
        let key = *w.fronts.keys().next().unwrap();
        let f = w.fronts.get_mut(&key).unwrap();
        f.actions = [true, false];
        f.commitments = [20.0, 10.0];
        f.path = Some([0, 2]);
        f.initiator = Some(0);
        let (_, edges) = w.fight().unwrap();
        assert_eq!(edges.len(), 1);
        assert!(edges[0].2 > 0.0);
        assert!(w.states[&key[1]].previous_damage > 0.0);
        let mut artifact = prescribed(&[0, 0, 2, 2], 2);
        artifact.config.severity_damage = SeverityDamage::MutualOnly;
        artifact.fronts.insert(key, w.fronts[&key].clone());
        artifact.fronts.get_mut(&key).unwrap().actions = [true, false];
        let (_, edges) = artifact.fight().unwrap();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].2, 0.0);
    }
    #[test]
    fn real_period_runs_decisions_damage_and_simultaneous_defender_priority() {
        let mut w = prescribed(&[0, 0, 2, 2], 2);
        w.config.attack_probability = 1.0;
        w.config.superiority_threshold = 1e-30;
        w.config.victory_threshold = 1e-30;
        w.config.defender_threshold = DefenderThreshold::SameThreshold;
        w.config.observation_periods = 1;
        w.run(1);
        assert_eq!(w.ledger.attacks, 2);
        assert_eq!(w.ledger.double_successes, 1);
        assert_eq!(w.ledger.conquests, 0);
        assert!(w.ledger.damage > 0.0);
        assert_eq!(w.outcome().unwrap().censored_wars.len(), 1);
    }
    #[test]
    fn artifact_profile_applies_certified_representable_corrections() {
        let c = GeosimConfig::artifact_2017();
        assert_eq!(c.count_boundary, CountBoundary::AtInitialization);
        assert_eq!(
            c.technology_inheritance,
            TechnologyInheritance::RetainCellThreshold
        );
        assert_eq!(c.initiation_guard, InitiationGuard::GlobalNoAction);
        assert_eq!(c.numerical_policy, NumericalPolicy::FloorZero);
        assert_eq!(
            GeosimConfig::default().count_boundary,
            CountBoundary::AfterInitialization
        );
    }
}
pub fn victory_probabilities(
    projected: [f64; 2],
    attacker: usize,
    config: &GeosimConfig,
) -> Result<[f64; 2], String> {
    let defender = 1 - attacker;
    let defender_threshold = if config.defender_threshold == DefenderThreshold::Reciprocal {
        1.0 / config.victory_threshold
    } else {
        config.victory_threshold
    };
    Ok([
        contest_probability(
            projected[attacker],
            projected[defender],
            config.victory_threshold,
            config.victory_exponent,
            config.numerical_policy,
        )?,
        contest_probability(
            projected[defender],
            projected[attacker],
            defender_threshold,
            config.victory_exponent,
            config.numerical_policy,
        )?,
    ])
}
#[cfg(test)]
mod defender_threshold_tests {
    use super::*;
    #[test]
    fn equal_commitments_give_source_defensive_advantage() {
        let p = victory_probabilities([1.0, 1.0], 0, &GeosimConfig::default()).unwrap();
        let attacker = 1.0 / 3486784402.0;
        assert!((p[0] - attacker).abs() < 1e-15);
        assert!((p[1] - (1.0 - attacker)).abs() < 1e-15);
    }
    #[test]
    fn attack_advantage_three_makes_both_victory_claims_half_likely() {
        assert_eq!(
            victory_probabilities([3.0, 1.0], 0, &GeosimConfig::default()).unwrap(),
            [0.5, 0.5]
        );
    }
}

pub fn losses_with_incidence(
    actions: [bool; 2],
    commitments: [f64; 2],
    projected: [f64; 2],
    fraction: f64,
    basis: DamageBasis,
    incidence: DamageIncidence,
) -> [f64; 2] {
    std::array::from_fn(|side| {
        let acting = if incidence == DamageIncidence::AttackedParty {
            1 - side
        } else {
            side
        };
        if actions[acting] {
            fraction
                * if basis == DamageBasis::OpponentProjected {
                    projected[1 - side]
                } else {
                    commitments[side]
                }
        } else {
            0.0
        }
    })
}
#[cfg(test)]
mod incidence_tests {
    use super::*;
    #[test]
    fn literal_acting_party_cost_changes_unilateral_recipient_independently_of_amount() {
        assert_eq!(
            losses_with_incidence(
                [true, false],
                [20.0, 10.0],
                [5.0, 2.0],
                0.1,
                DamageBasis::OwnCommitment,
                DamageIncidence::ActingParty
            ),
            [2.0, 0.0]
        );
        assert_eq!(
            losses_with_incidence(
                [true, false],
                [20.0, 10.0],
                [5.0, 2.0],
                0.1,
                DamageBasis::OpponentProjected,
                DamageIncidence::AttackedParty
            ),
            [0.0, 0.5]
        );
        assert_eq!(
            losses_with_incidence(
                [true, true],
                [20.0, 10.0],
                [5.0, 2.0],
                0.1,
                DamageBasis::OwnCommitment,
                DamageIncidence::ActingParty
            ),
            [2.0, 1.0]
        );
    }
}
