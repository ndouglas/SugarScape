use super::{
    Config, DecisionAction, DecisionObservation, Error, FrozenListener, Policy, Profile,
    StrategicObservation,
};
use serde::{Deserialize, Serialize};

/// Exact private-world distribution. Private realizations are never controller inputs.
#[derive(Clone, Debug)]
pub struct Distribution {
    config: Config,
    denominator: u64,
    pub(crate) worlds: Vec<World>,
}
impl Distribution {
    pub fn config(&self) -> &Config {
        &self.config
    }
    pub fn denominator(&self) -> u64 {
        self.denominator
    }
}
#[derive(Clone, Debug)]
pub(crate) struct World {
    pub(crate) calibration_truth: bool,
    pub(crate) live_truth: bool,
    pub(crate) fixed_profile: Profile,
    pub(crate) calibration_signals: [bool; 2],
    pub(crate) live_signals: [bool; 2],
    pub(crate) mass: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExactPosterior {
    pub numerator: u64,
    pub denominator: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HistoryMass {
    pub observation: DecisionObservation,
    pub total_mass: u64,
    pub true_mass: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HistoryEvaluation {
    pub observation: DecisionObservation,
    pub total_mass: u64,
    pub true_mass: u64,
    /// Actual-policy conditional probability, absent for zero-mass histories.
    pub reference_posterior: Option<ExactPosterior>,
    pub reference_action: Option<DecisionAction>,
    pub listener_action: Option<DecisionAction>,
    pub regret_numerator: i64,
    pub reference_regret_numerator: i64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Evaluation {
    pub payoff_numerator: i64,
    pub utility_numerator: i64,
    pub optimal_numerator: i64,
    pub regret_numerator: i64,
    pub reference_regret_numerator: i64,
    pub denominator: u64,
    pub intervention_mass: u64,
    pub correct_intervention_mass: u64,
    pub false_intervention_mass: u64,
    pub missed_beneficial_intervention_mass: u64,
    /// Successful evaluations have no error mass; unsupported assumed histories return Err.
    pub error_mass: u64,
    pub calibration_truth_agreement_mass: u64,
    pub live_truth_agreement_mass: u64,
    pub calibration_signal_opposition_mass: u64,
    pub live_signal_opposition_mass: u64,
    pub histories: Vec<HistoryEvaluation>,
}
pub(crate) fn checked_add(target: &mut i64, value: i64) -> Result<(), Error> {
    *target = target.checked_add(value).ok_or(Error::ArithmeticOverflow)?;
    Ok(())
}
fn add_mass(target: &mut u64, value: u64) -> Result<(), Error> {
    *target = target.checked_add(value).ok_or(Error::ArithmeticOverflow)?;
    Ok(())
}
pub fn enumerate(config: &Config) -> Result<Distribution, Error> {
    config.validate()?;
    let q = &config.accuracy;
    let rho = &config.fixed_copy_prior;
    let denominator = u64::from(q.denominator)
        .checked_pow(4)
        .and_then(|mass| mass.checked_mul(u64::from(rho.denominator)))
        .and_then(|mass| mass.checked_mul(4))
        .ok_or(Error::ArithmeticOverflow)?;
    let mut worlds = Vec::with_capacity(128);
    let mut total = 0;
    for bits in 0..128u16 {
        let c = bits & 64 != 0;
        let t = bits & 32 != 0;
        let copy = bits & 16 != 0;
        let c_signals = [bits & 8 != 0, bits & 4 != 0];
        let t_signals = [bits & 2 != 0, bits & 1 != 0];
        let mut mass = u64::from(if copy {
            rho.numerator
        } else {
            rho.denominator - rho.numerator
        });
        for (truth, signal) in [
            (c, c_signals[0]),
            (c, c_signals[1]),
            (t, t_signals[0]),
            (t, t_signals[1]),
        ] {
            mass = mass
                .checked_mul(u64::from(if truth == signal {
                    q.numerator
                } else {
                    q.denominator - q.numerator
                }))
                .ok_or(Error::ArithmeticOverflow)?;
        }
        add_mass(&mut total, mass)?;
        worlds.push(World {
            calibration_truth: c,
            live_truth: t,
            fixed_profile: if copy { Profile::Copy } else { Profile::Invert },
            calibration_signals: c_signals,
            live_signals: t_signals,
            mass,
        });
    }
    if total != denominator {
        return Err(Error::InvalidDistribution("world masses do not normalize"));
    }
    Ok(Distribution {
        config: config.clone(),
        denominator,
        worlds,
    })
}
pub(crate) fn observation(config: &Config, index: usize) -> DecisionObservation {
    DecisionObservation {
        rules: config.clone(),
        calibration_truth: index & 16 != 0,
        calibration_reports: [index & 8 != 0, index & 4 != 0],
        live_reports: [index & 2 != 0, index & 1 != 0],
    }
}
pub(crate) fn history_index(
    truth: bool,
    calibration_reports: [bool; 2],
    live_reports: [bool; 2],
) -> usize {
    (usize::from(truth) << 4)
        | (usize::from(calibration_reports[0]) << 3)
        | (usize::from(calibration_reports[1]) << 2)
        | (usize::from(live_reports[0]) << 1)
        | usize::from(live_reports[1])
}
pub(crate) fn fixed_report(profile: Profile, signal: bool) -> bool {
    super::super::testimony_game::report(profile, signal)
}
fn reports(
    config: &Config,
    world: &World,
    policy: &Policy,
) -> Result<([bool; 2], [bool; 2]), Error> {
    let calibration = policy.report(&StrategicObservation {
        rules: config.clone(),
        signal: world.calibration_signals[0],
        calibration_signal: None,
        calibration_reports: None,
        calibration_truth: None,
    })?;
    let calibration_reports = [
        calibration,
        fixed_report(world.fixed_profile, world.calibration_signals[1]),
    ];
    let live = policy.report(&StrategicObservation {
        rules: config.clone(),
        signal: world.live_signals[0],
        calibration_signal: Some(world.calibration_signals[0]),
        calibration_reports: Some(calibration_reports),
        calibration_truth: Some(world.calibration_truth),
    })?;
    Ok((
        calibration_reports,
        [
            live,
            fixed_report(world.fixed_profile, world.live_signals[1]),
        ],
    ))
}
struct CandidateMasses {
    histories: Vec<HistoryMass>,
    calibration_truth_agreement: u64,
    live_truth_agreement: u64,
    calibration_signal_opposition: u64,
    live_signal_opposition: u64,
}
fn candidate_masses(d: &Distribution, policy: &Policy) -> Result<CandidateMasses, Error> {
    Policy::new(policy.bits)?;
    let mut result = CandidateMasses {
        histories: (0..32)
            .map(|index| HistoryMass {
                observation: observation(&d.config, index),
                total_mass: 0,
                true_mass: 0,
            })
            .collect(),
        calibration_truth_agreement: 0,
        live_truth_agreement: 0,
        calibration_signal_opposition: 0,
        live_signal_opposition: 0,
    };
    for world in &d.worlds {
        let (calibration, live) = reports(&d.config, world, policy)?;
        let h = &mut result.histories[history_index(world.calibration_truth, calibration, live)];
        add_mass(&mut h.total_mass, world.mass)?;
        if world.live_truth {
            add_mass(&mut h.true_mass, world.mass)?;
        }
        if calibration[0] == world.calibration_truth {
            add_mass(&mut result.calibration_truth_agreement, world.mass)?;
        }
        if live[0] == world.live_truth {
            add_mass(&mut result.live_truth_agreement, world.mass)?;
        }
        if calibration[0] != world.calibration_signals[0] {
            add_mass(&mut result.calibration_signal_opposition, world.mass)?;
        }
        if live[0] != world.live_signals[0] {
            add_mass(&mut result.live_signal_opposition, world.mass)?;
        }
    }
    Ok(result)
}
pub fn histories(d: &Distribution, policy: &Policy) -> Result<Vec<HistoryMass>, Error> {
    Ok(candidate_masses(d, policy)?.histories)
}
pub fn evaluate(
    d: &Distribution,
    policy: &Policy,
    listener: &FrozenListener,
) -> Result<Evaluation, Error> {
    let masses = candidate_masses(d, policy)?;
    let mut e = Evaluation {
        payoff_numerator: 0,
        utility_numerator: 0,
        optimal_numerator: 0,
        regret_numerator: 0,
        reference_regret_numerator: 0,
        denominator: d.denominator,
        intervention_mass: 0,
        correct_intervention_mass: 0,
        false_intervention_mass: 0,
        missed_beneficial_intervention_mass: 0,
        error_mass: 0,
        calibration_truth_agreement_mass: masses.calibration_truth_agreement,
        live_truth_agreement_mass: masses.live_truth_agreement,
        calibration_signal_opposition_mass: masses.calibration_signal_opposition,
        live_signal_opposition_mass: masses.live_signal_opposition,
        histories: Vec::with_capacity(32),
    };
    for h in masses.histories {
        if h.total_mass == 0 {
            e.histories.push(HistoryEvaluation {
                observation: h.observation,
                total_mass: 0,
                true_mass: 0,
                reference_posterior: None,
                reference_action: None,
                listener_action: None,
                regret_numerator: 0,
                reference_regret_numerator: 0,
            });
            continue;
        }
        let total = i64::try_from(h.total_mass).map_err(|_| Error::ArithmeticOverflow)?;
        let yes = i64::try_from(h.true_mass).map_err(|_| Error::ArithmeticOverflow)?;
        let gain = yes
            .checked_mul(2)
            .and_then(|v| v.checked_sub(total))
            .ok_or(Error::ArithmeticOverflow)?;
        let reference_action = if gain > 0 {
            DecisionAction::Intervene
        } else {
            DecisionAction::Abstain
        };
        let action = listener.decide(&h.observation)?;
        let payoff = if action == DecisionAction::Intervene {
            add_mass(&mut e.intervention_mass, h.total_mass)?;
            add_mass(&mut e.correct_intervention_mass, h.true_mass)?;
            add_mass(&mut e.false_intervention_mass, h.total_mass - h.true_mass)?;
            gain
        } else {
            add_mass(&mut e.missed_beneficial_intervention_mass, h.true_mass)?;
            0
        };
        let utility = yes
            .checked_mul(i64::from(d.config.strategic_utility.utility(true, action)))
            .and_then(|v| {
                (total - yes)
                    .checked_mul(i64::from(d.config.strategic_utility.utility(false, action)))
                    .and_then(|w| v.checked_add(w))
            })
            .ok_or(Error::ArithmeticOverflow)?;
        let optimal = gain.max(0);
        let reference_payoff = if reference_action == DecisionAction::Intervene {
            gain
        } else {
            0
        };
        checked_add(&mut e.payoff_numerator, payoff)?;
        checked_add(&mut e.utility_numerator, utility)?;
        checked_add(&mut e.optimal_numerator, optimal)?;
        checked_add(
            &mut e.reference_regret_numerator,
            optimal - reference_payoff,
        )?;
        e.histories.push(HistoryEvaluation {
            observation: h.observation,
            total_mass: h.total_mass,
            true_mass: h.true_mass,
            reference_posterior: Some(ExactPosterior {
                numerator: h.true_mass,
                denominator: h.total_mass,
            }),
            reference_action: Some(reference_action),
            listener_action: Some(action),
            regret_numerator: optimal - payoff,
            reference_regret_numerator: optimal - reference_payoff,
        });
    }
    e.regret_numerator = e
        .optimal_numerator
        .checked_sub(e.payoff_numerator)
        .ok_or(Error::ArithmeticOverflow)?;
    Ok(e)
}
