use super::{Catalog, Error, Model, Ratio, SignedRatio};
use crate::deduction::strategic_reporting::{
    Config, DecisionAction, DecisionObservation, FrozenListener, Policy,
};
use serde::{Deserialize, Serialize};
/// Strategy listeners provide exact beliefs; frozen legacy listeners provide actions only.
pub enum EvaluatedListener {
    Strategy(Model),
    Legacy(FrozenListener),
}
/// One complete public history, retaining its mass under the actual distribution.
/// Zero-mass rows have no action or posterior and are not marked unsupported.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvaluatedHistory {
    pub observation: DecisionObservation,
    pub actual_mass: u64,
    pub actual_true_mass: u64,
    pub actual_posterior: Option<Ratio>,
    pub listener_posterior: Option<Ratio>,
    pub belief_error: Option<SignedRatio>,
    pub action: Option<DecisionAction>,
    pub unsupported: bool,
}
/// Supported sums retain the original distribution denominator. Unconditional
/// scores are absent whenever any positive actual mass is unsupported. The
/// maximum error ranges over supported, positive-mass strategy beliefs only.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Evaluation {
    pub denominator: u64,
    pub supported_mass: u64,
    pub unsupported_mass: u64,
    pub supported_payoff_numerator: i64,
    pub supported_regret_numerator: i64,
    pub supported_reporter_utility_numerator: i64,
    pub payoff: Option<SignedRatio>,
    pub decision_regret: Option<Ratio>,
    pub reporter_utility: Option<SignedRatio>,
    pub maximum_belief_error: Option<Ratio>,
    pub histories: Vec<EvaluatedHistory>,
}

/// Evaluate transfer against one actual policy and its informed action reference.
pub fn evaluate_fixed(
    config: &Config,
    actual: &Policy,
    listener: &EvaluatedListener,
) -> Result<Evaluation, Error> {
    let distribution = crate::deduction::strategic_reporting::enumerate(config)?;
    let rows = crate::deduction::strategic_reporting::histories(&distribution, actual)?;
    evaluate(config, distribution.denominator(), rows, listener)
}

/// Evaluate one actual mixture, retaining each sampled policy across both phases.
pub fn evaluate_mixture(
    config: &Config,
    actual: &Catalog,
    listener: &EvaluatedListener,
) -> Result<Evaluation, Error> {
    let (denominator, policies) = super::model::weighted_policy_masses(config, actual)?;
    let mut rows = policies[0].histories.clone();
    for policy in &policies[1..] {
        for (row, contribution) in rows.iter_mut().zip(&policy.histories) {
            super::model::add_mass(&mut row.total_mass, contribution.total_mass)?;
            super::model::add_mass(&mut row.true_mass, contribution.true_mass)?;
        }
    }
    evaluate(config, denominator, rows, listener)
}

fn decide(
    listener: &EvaluatedListener,
    view: &DecisionObservation,
) -> Result<(DecisionAction, Option<Ratio>), Error> {
    match listener {
        EvaluatedListener::Strategy(model) => model
            .decide(view)
            .map(|decision| (decision.action, Some(decision.posterior_true))),
        EvaluatedListener::Legacy(legacy) => Ok((legacy.decide(view)?, None)),
    }
}

fn belief_error(model: &Ratio, actual: &Ratio) -> Result<SignedRatio, Error> {
    let denominator = model
        .denominator
        .checked_mul(actual.denominator)
        .ok_or(Error::ArithmeticOverflow)?;
    let model_true = i128::from(model.numerator)
        .checked_mul(i128::from(actual.denominator))
        .ok_or(Error::ArithmeticOverflow)?;
    let actual_true = i128::from(actual.numerator)
        .checked_mul(i128::from(model.denominator))
        .ok_or(Error::ArithmeticOverflow)?;
    let numerator = i64::try_from(
        model_true
            .checked_sub(actual_true)
            .ok_or(Error::ArithmeticOverflow)?,
    )
    .map_err(|_| Error::ArithmeticOverflow)?;
    Ok(SignedRatio {
        numerator,
        denominator,
    })
}

fn update_maximum(maximum: &mut Option<Ratio>, error: &SignedRatio) -> Result<(), Error> {
    let candidate = Ratio {
        numerator: error.numerator.unsigned_abs(),
        denominator: error.denominator,
    };
    let greater = match maximum {
        None => true,
        Some(current) => {
            let left = i128::from(candidate.numerator)
                .checked_mul(i128::from(current.denominator))
                .ok_or(Error::ArithmeticOverflow)?;
            let right = i128::from(current.numerator)
                .checked_mul(i128::from(candidate.denominator))
                .ok_or(Error::ArithmeticOverflow)?;
            left > right
        }
    };
    if greater {
        *maximum = Some(candidate);
    }
    Ok(())
}

fn add_score(target: &mut i64, value: i64) -> Result<(), Error> {
    *target = target.checked_add(value).ok_or(Error::ArithmeticOverflow)?;
    Ok(())
}

fn evaluate(
    config: &Config,
    denominator: u64,
    rows: Vec<crate::deduction::strategic_reporting::HistoryMass>,
    listener: &EvaluatedListener,
) -> Result<Evaluation, Error> {
    let mut evaluation = Evaluation {
        denominator,
        supported_mass: 0,
        unsupported_mass: 0,
        supported_payoff_numerator: 0,
        supported_regret_numerator: 0,
        supported_reporter_utility_numerator: 0,
        payoff: None,
        decision_regret: None,
        reporter_utility: None,
        maximum_belief_error: None,
        histories: Vec::with_capacity(rows.len()),
    };
    for row in rows {
        let mut scored = EvaluatedHistory {
            observation: row.observation,
            actual_mass: row.total_mass,
            actual_true_mass: row.true_mass,
            actual_posterior: None,
            listener_posterior: None,
            belief_error: None,
            action: None,
            unsupported: false,
        };
        // Zero actual mass does not require a listener decision or imply support failure.
        if row.total_mass == 0 {
            evaluation.histories.push(scored);
            continue;
        }
        scored.actual_posterior = Some(Ratio {
            numerator: row.true_mass,
            denominator: row.total_mass,
        });
        match decide(listener, &scored.observation) {
            Err(Error::ZeroEvidence)
            | Err(Error::Existing(crate::deduction::strategic_reporting::Error::LegacyListener(
                crate::deduction::testimony_game::Error::Inference(
                    crate::deduction::TestimonyError::ZeroEvidence,
                ),
            ))) => {
                scored.unsupported = true;
                super::model::add_mass(&mut evaluation.unsupported_mass, row.total_mass)?;
            }
            Err(error) => return Err(error),
            Ok((action, posterior)) => {
                super::model::add_mass(&mut evaluation.supported_mass, row.total_mass)?;
                let mass = i64::try_from(row.total_mass).map_err(|_| Error::ArithmeticOverflow)?;
                let true_mass =
                    i64::try_from(row.true_mass).map_err(|_| Error::ArithmeticOverflow)?;
                let gain = true_mass
                    .checked_mul(2)
                    .and_then(|value| value.checked_sub(mass))
                    .ok_or(Error::ArithmeticOverflow)?;
                let payoff = if action == DecisionAction::Intervene {
                    gain
                } else {
                    0
                };
                add_score(&mut evaluation.supported_payoff_numerator, payoff)?;
                add_score(
                    &mut evaluation.supported_regret_numerator,
                    gain.max(0) - payoff,
                )?;
                let utility = true_mass * i64::from(config.strategic_utility.utility(true, action))
                    + (mass - true_mass)
                        * i64::from(config.strategic_utility.utility(false, action));
                add_score(
                    &mut evaluation.supported_reporter_utility_numerator,
                    utility,
                )?;
                if let Some(posterior) = &posterior {
                    let error = belief_error(posterior, scored.actual_posterior.as_ref().unwrap())?;
                    update_maximum(&mut evaluation.maximum_belief_error, &error)?;
                    scored.belief_error = Some(error);
                }
                scored.listener_posterior = posterior;
                scored.action = Some(action);
            }
        }
        evaluation.histories.push(scored);
    }
    if evaluation
        .supported_mass
        .checked_add(evaluation.unsupported_mass)
        != Some(denominator)
    {
        return Err(Error::InvalidObservation(
            "actual history masses do not normalize",
        ));
    }
    if evaluation.unsupported_mass == 0 {
        evaluation.payoff = Some(SignedRatio {
            numerator: evaluation.supported_payoff_numerator,
            denominator,
        });
        evaluation.decision_regret = Some(Ratio {
            numerator: u64::try_from(evaluation.supported_regret_numerator)
                .map_err(|_| Error::ArithmeticOverflow)?,
            denominator,
        });
        evaluation.reporter_utility = Some(SignedRatio {
            numerator: evaluation.supported_reporter_utility_numerator,
            denominator,
        });
    }
    Ok(evaluation)
}
