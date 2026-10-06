//! Expected payoff and informed-reference comparisons under public game rules.
//! Minimax protection concerns expectation in this game, not each realized outcome.
use super::scoring::{history_contribution, validate_history_masses};
use super::{AuditEvaluation, Error, FrozenActions, Score};
use crate::deduction::strategic_reporting::{
    self, enumerate, histories, DecisionAction, Distribution, ExactPosterior, FrozenListener,
    HistoryEvaluation, HistoryMass, Listener, Policy,
};
use crate::deduction::strategy_inference::{Catalog, Ratio, SignedRatio};

#[derive(Default)]
struct PhaseStatistics {
    calibration_truth_agreement: u64,
    live_truth_agreement: u64,
    calibration_signal_opposition: u64,
    live_signal_opposition: u64,
}

impl PhaseStatistics {
    fn for_policy(distribution: &Distribution, policy: &Policy) -> Result<Self, Error> {
        // These statistics do not depend on receiver actions. Extract only the
        // phase fields; the passive adapter never decides for the audit target.
        let evaluation = strategic_reporting::evaluate(
            distribution,
            policy,
            &FrozenListener {
                algorithm: Listener::Passive,
                assumed_copy_prior: distribution.config().fixed_copy_prior.clone(),
            },
        )?;
        Ok(Self {
            calibration_truth_agreement: evaluation.calibration_truth_agreement_mass,
            live_truth_agreement: evaluation.live_truth_agreement_mass,
            calibration_signal_opposition: evaluation.calibration_signal_opposition_mass,
            live_signal_opposition: evaluation.live_signal_opposition_mass,
        })
    }

    fn add_weighted(&mut self, other: &Self, weight: u64) -> Result<(), Error> {
        add_weighted_mass(
            &mut self.calibration_truth_agreement,
            other.calibration_truth_agreement,
            weight,
        )?;
        add_weighted_mass(
            &mut self.live_truth_agreement,
            other.live_truth_agreement,
            weight,
        )?;
        add_weighted_mass(
            &mut self.calibration_signal_opposition,
            other.calibration_signal_opposition,
            weight,
        )?;
        add_weighted_mass(
            &mut self.live_signal_opposition,
            other.live_signal_opposition,
            weight,
        )
    }
}

/// Evaluate immutable receiver actions against one actual reporting policy.
/// Actual policy evidence is used by the informed reference, never the receiver.
pub fn evaluate_fixed(actions: &FrozenActions, actual: &Policy) -> Result<AuditEvaluation, Error> {
    Policy::new(actual.bits)?;
    let distribution = enumerate(actions.config())?;
    let rows = histories(&distribution, actual)?;
    let phases = PhaseStatistics::for_policy(&distribution, actual)?;
    evaluate_histories(actions, distribution.denominator(), rows, phases)
}

/// Evaluate one actual population, pooling its history evidence before inferring
/// reference actions. The sampled policy remains fixed across both phases.
pub fn evaluate_mixture(
    actions: &FrozenActions,
    actual: &Catalog,
) -> Result<AuditEvaluation, Error> {
    let distribution = enumerate(actions.config())?;
    let denominator = distribution
        .denominator()
        .checked_mul(u64::from(actual.total_weight()))
        .ok_or(Error::ArithmeticOverflow)?;
    let mut rows: Vec<_> = actions
        .rows()
        .iter()
        .map(|row| HistoryMass {
            observation: row.observation.clone(),
            total_mass: 0,
            true_mass: 0,
        })
        .collect();
    let mut phases = PhaseStatistics::default();
    for entry in actual.entries() {
        let weight = u64::from(entry.weight);
        if weight == 0 {
            continue;
        }
        for (pooled, contribution) in rows
            .iter_mut()
            .zip(histories(&distribution, &entry.policy)?)
        {
            add_weighted_mass(&mut pooled.total_mass, contribution.total_mass, weight)?;
            add_weighted_mass(&mut pooled.true_mass, contribution.true_mass, weight)?;
        }
        phases.add_weighted(
            &PhaseStatistics::for_policy(&distribution, &entry.policy)?,
            weight,
        )?;
    }
    evaluate_histories(actions, denominator, rows, phases)
}

fn add_weighted_mass(target: &mut u64, mass: u64, weight: u64) -> Result<(), Error> {
    let contribution = mass.checked_mul(weight).ok_or(Error::ArithmeticOverflow)?;
    *target = target
        .checked_add(contribution)
        .ok_or(Error::ArithmeticOverflow)?;
    Ok(())
}

fn add_score(target: &mut i64, contribution: i64) -> Result<(), Error> {
    *target = target
        .checked_add(contribution)
        .ok_or(Error::ArithmeticOverflow)?;
    Ok(())
}

fn evaluate_histories(
    actions: &FrozenActions,
    denominator: u64,
    rows: Vec<HistoryMass>,
    phases: PhaseStatistics,
) -> Result<AuditEvaluation, Error> {
    // Validate all 32 rows and their normalization before skipping zero evidence.
    validate_history_masses(actions.config(), denominator, &rows)?;
    let mut evaluation = AuditEvaluation {
        payoff_numerator: 0,
        utility_numerator: 0,
        optimal_numerator: 0,
        regret_numerator: 0,
        reference_regret_numerator: 0,
        denominator,
        intervention_mass: 0,
        correct_intervention_mass: 0,
        false_intervention_mass: 0,
        missed_beneficial_intervention_mass: 0,
        error_mass: 0,
        calibration_truth_agreement_mass: phases.calibration_truth_agreement,
        live_truth_agreement_mass: phases.live_truth_agreement,
        calibration_signal_opposition_mass: phases.calibration_signal_opposition,
        live_signal_opposition_mass: phases.live_signal_opposition,
        histories: Vec::with_capacity(32),
    };
    for (history, action_row) in rows.into_iter().zip(actions.rows()) {
        let mut row = HistoryEvaluation {
            observation: history.observation.clone(),
            total_mass: history.total_mass,
            true_mass: history.true_mass,
            reference_posterior: None,
            reference_action: None,
            listener_action: None,
            regret_numerator: 0,
            reference_regret_numerator: 0,
        };
        if history.total_mass == 0 {
            evaluation.histories.push(row);
            continue;
        }
        let action = action_row.decision.action;
        let contribution = history_contribution(&history, action)?;
        row.reference_posterior = Some(ExactPosterior {
            numerator: history.true_mass,
            denominator: history.total_mass,
        });
        row.reference_action = Some(if contribution.gain > 0 {
            DecisionAction::Intervene
        } else {
            DecisionAction::Abstain
        });
        row.listener_action = Some(action);
        row.regret_numerator = contribution.regret;
        if action == DecisionAction::Intervene {
            add_weighted_mass(&mut evaluation.intervention_mass, history.total_mass, 1)?;
            add_weighted_mass(
                &mut evaluation.correct_intervention_mass,
                history.true_mass,
                1,
            )?;
            add_weighted_mass(
                &mut evaluation.false_intervention_mass,
                history.total_mass - history.true_mass,
                1,
            )?;
        } else {
            add_weighted_mass(
                &mut evaluation.missed_beneficial_intervention_mass,
                history.true_mass,
                1,
            )?;
        }
        add_score(&mut evaluation.payoff_numerator, contribution.payoff)?;
        add_score(&mut evaluation.utility_numerator, contribution.utility)?;
        add_score(&mut evaluation.optimal_numerator, contribution.optimal)?;
        add_score(&mut evaluation.regret_numerator, contribution.regret)?;
        evaluation.histories.push(row);
    }
    Score {
        rules: actions.config().clone(),
        denominator,
        payoff_numerator: evaluation.payoff_numerator,
        utility_numerator: evaluation.utility_numerator,
    }
    .validate()?;
    Ok(evaluation)
}

fn payoff_difference(left: &Score, right: &Score) -> Result<SignedRatio, Error> {
    left.validate()?;
    right.validate()?;
    if left.rules != right.rules {
        return Err(Error::InvalidRules("comparison score rules differ"));
    }
    let left_numerator = i128::from(left.payoff_numerator)
        .checked_mul(i128::from(right.denominator))
        .ok_or(Error::ArithmeticOverflow)?;
    let right_numerator = i128::from(right.payoff_numerator)
        .checked_mul(i128::from(left.denominator))
        .ok_or(Error::ArithmeticOverflow)?;
    let numerator = left_numerator
        .checked_sub(right_numerator)
        .ok_or(Error::ArithmeticOverflow)?;
    let denominator = i128::from(left.denominator)
        .checked_mul(i128::from(right.denominator))
        .ok_or(Error::ArithmeticOverflow)?;
    Ok(SignedRatio {
        numerator: i64::try_from(numerator).map_err(|_| Error::ArithmeticOverflow)?,
        denominator: u64::try_from(denominator).map_err(|_| Error::ArithmeticOverflow)?,
    })
}

/// F minus a targeted worst-case receiver payoff. This differs from regret
/// against a reference that knows the actual policy. A negative shortfall fails.
pub fn guarantee_shortfall(fixed: &Score, worst: &Score) -> Result<Ratio, Error> {
    let difference = payoff_difference(fixed, worst)?;
    if difference.numerator < 0 {
        return Err(Error::InvalidReport(
            "guarantee shortfall must be nonnegative",
        ));
    }
    Ok(Ratio {
        numerator: u64::try_from(difference.numerator).map_err(|_| Error::ArithmeticOverflow)?,
        denominator: difference.denominator,
    })
}

/// Nominal receiver payoff minus fixed-only payoff for the same public rules.
/// This signed population comparison is separate from guarantee shortfall.
pub fn nominal_difference(nominal: &Score, fixed: &Score) -> Result<SignedRatio, Error> {
    payoff_difference(nominal, fixed)
}
