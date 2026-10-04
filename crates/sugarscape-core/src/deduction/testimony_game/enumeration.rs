use super::{
    Config, DecisionAction, DecisionObservation, Error, Genome, Listener, ListenerDecision,
};
use crate::deduction::FieldError;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HistoryMass {
    pub observation: DecisionObservation,
    pub total_mass: u64,
    pub true_mass: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Distribution {
    pub config: Config,
    pub denominator: u64,
    pub histories: Vec<HistoryMass>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HistoryEvaluation {
    pub observation: DecisionObservation,
    pub total_mass: u64,
    pub true_mass: u64,
    pub reference_posterior: f64,
    pub decision: ListenerDecision,
    pub conditional_regret: f64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Evaluation {
    pub payoff_numerator: i64,
    pub optimal_numerator: i64,
    pub regret_numerator: i64,
    pub denominator: u64,
    pub intervention_mass: u64,
    pub correct_intervention_mass: u64,
    pub incorrect_intervention_mass: u64,
    pub posterior_max_error: Option<f64>,
    pub posterior_squared_error: Option<f64>,
    pub brier: Option<f64>,
    pub histories: Vec<HistoryEvaluation>,
}
fn denominator(c: &Config) -> u64 {
    4 * u64::from(c.copy_prior.denominator).pow(2) * u64::from(c.accuracy.denominator).pow(4)
}
pub fn enumerate(config: &Config) -> Result<Distribution, Error> {
    config.validate()?;
    let mut masses: BTreeMap<[bool; 5], (u64, u64)> = BTreeMap::new();
    let q = &config.accuracy;
    let rho = &config.copy_prior;
    for bits in 0..256u16 {
        let v: Vec<bool> = (0..8).map(|i| bits & (1 << (7 - i)) != 0).collect();
        let [c, t, p0, p1, c0, c1, t0, t1]: [bool; 8] =
            v.try_into().expect("eight Boolean variables");
        let mut mass = 1u64;
        for copy in [p0, p1] {
            mass *= u64::from(if copy {
                rho.numerator
            } else {
                rho.denominator - rho.numerator
            });
        }
        for (truth, signal) in [(c, c0), (c, c1), (t, t0), (t, t1)] {
            mass *= u64::from(if truth == signal {
                q.numerator
            } else {
                q.denominator - q.numerator
            });
        }
        if mass == 0 {
            continue;
        }
        let key = [c, c0 == p0, c1 == p1, t0 == p0, t1 == p1];
        let entry = masses.entry(key).or_default();
        entry.0 += mass;
        if t {
            entry.1 += mass;
        }
    }
    Ok(Distribution {
        config: config.clone(),
        denominator: denominator(config),
        histories: masses
            .into_iter()
            .map(
                |([c, c0, c1, t0, t1], (total_mass, true_mass))| HistoryMass {
                    observation: DecisionObservation {
                        rules: config.clone(),
                        calibration_truth: c,
                        calibration_reports: [c0, c1],
                        live_reports: [t0, t1],
                    },
                    total_mass,
                    true_mass,
                },
            )
            .collect(),
    })
}
fn invalid(message: &str) -> Error {
    Error::InvalidConfig(vec![FieldError::new("distribution", message)])
}
fn validate(d: &Distribution) -> Result<(), Error> {
    d.config.validate()?;
    if d.denominator != denominator(&d.config) {
        return Err(invalid("denominator must match public probabilities"));
    }
    let mut total = 0u64;
    let mut truth = 0u64;
    let mut keys = BTreeSet::new();
    for h in &d.histories {
        h.observation.rules.validate()?;
        if h.observation.rules != d.config || h.total_mass == 0 || h.true_mass > h.total_mass {
            return Err(invalid("history rules or masses are inconsistent"));
        }
        let v = &h.observation;
        if !keys.insert((v.calibration_truth, v.calibration_reports, v.live_reports)) {
            return Err(invalid("duplicate observation"));
        }
        total = total
            .checked_add(h.total_mass)
            .ok_or_else(|| invalid("total mass overflow"))?;
        truth = truth
            .checked_add(h.true_mass)
            .ok_or_else(|| invalid("true mass overflow"))?;
    }
    if total != d.denominator || truth != d.denominator / 2 {
        return Err(invalid(
            "masses must normalize with the independent half-truth prior",
        ));
    }
    Ok(())
}
fn gain(h: &HistoryMass) -> i64 {
    2 * h.true_mass as i64 - h.total_mass as i64
}
pub fn evaluate(d: &Distribution, listener: &Listener) -> Result<Evaluation, Error> {
    validate(d)?;
    let mut e = Evaluation {
        payoff_numerator: 0,
        optimal_numerator: 0,
        regret_numerator: 0,
        denominator: d.denominator,
        intervention_mass: 0,
        correct_intervention_mass: 0,
        incorrect_intervention_mass: 0,
        posterior_max_error: None,
        posterior_squared_error: None,
        brier: None,
        histories: Vec::with_capacity(d.histories.len()),
    };
    for h in &d.histories {
        let decision = listener.decide(&h.observation)?;
        let reference = h.true_mass as f64 / h.total_mass as f64;
        let optimum = gain(h).max(0);
        let chosen = if decision.action == DecisionAction::Intervene {
            e.intervention_mass += h.total_mass;
            e.correct_intervention_mass += h.true_mass;
            e.incorrect_intervention_mass += h.total_mass - h.true_mass;
            gain(h)
        } else {
            0
        };
        e.payoff_numerator += chosen;
        e.optimal_numerator += optimum;
        if let Some(p) = decision.posterior_true {
            if !p.is_finite() || !(0.0..=1.0).contains(&p) {
                return Err(invalid("posterior must be finite and in range"));
            }
            let error = (p - reference).abs();
            let weight = h.total_mass as f64 / d.denominator as f64;
            e.posterior_max_error = Some(e.posterior_max_error.unwrap_or(0.0).max(error));
            *e.posterior_squared_error.get_or_insert(0.0) += weight * error.powi(2);
            *e.brier.get_or_insert(0.0) += (h.true_mass as f64 * (1.0 - p).powi(2)
                + (h.total_mass - h.true_mass) as f64 * p.powi(2))
                / d.denominator as f64;
        }
        e.histories.push(HistoryEvaluation {
            observation: h.observation.clone(),
            total_mass: h.total_mass,
            true_mass: h.true_mass,
            reference_posterior: reference,
            decision,
            conditional_regret: (optimum - chosen) as f64 / h.total_mass as f64,
        });
    }
    e.regret_numerator = e.optimal_numerator - e.payoff_numerator;
    Ok(e)
}
pub fn fitness(d: &Distribution, g: &Genome) -> Result<i64, Error> {
    validate(d)?;
    g.validate()?;
    let listener = Listener::Evolved(g.clone());
    let mut payoff = 0;
    for h in &d.histories {
        if listener.decide(&h.observation)?.action == DecisionAction::Intervene {
            payoff += gain(h);
        }
    }
    Ok(payoff)
}
