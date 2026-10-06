use super::{history_index, validate_rules, Error, FrozenActions};
use crate::deduction::strategic_reporting::{
    enumerate, histories, Config, DecisionAction, Distribution, HistoryMass, Policy,
};
use serde::{Deserialize, Serialize};

/// Exact receiver payoff and opposed reporter utility under these public rules.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Score {
    pub rules: Config,
    pub denominator: u64,
    pub payoff_numerator: i64,
    pub utility_numerator: i64,
}

impl Score {
    pub fn validate(&self) -> Result<(), Error> {
        validate_rules(&self.rules)?;
        validate_denominator(self.denominator)?;
        // Uniform T and intervene/abstain payoffs bound either sign by D/2.
        // Widen before taking abs/negating: public callers can forge i64::MIN.
        let payoff = i128::from(self.payoff_numerator);
        if payoff.abs() * 2 > i128::from(self.denominator) {
            return Err(Error::InvalidReport(
                "score payoff exceeds half its denominator",
            ));
        }
        if i128::from(self.utility_numerator) != -payoff {
            return Err(Error::InvalidReport(
                "score utility is not opposed to payoff",
            ));
        }
        Ok(())
    }
}

fn validate_denominator(denominator: u64) -> Result<(), Error> {
    // Maximum public-world D is 2^22; a catalog has total weight at most 256.
    if denominator == 0 || denominator > 1 << 30 {
        return Err(Error::InvalidReport("score denominator requires 1..=2^30"));
    }
    Ok(())
}

/// Validate complete public evidence before scoring or mixture conditioning.
pub(super) fn validate_history_masses(
    config: &Config,
    denominator: u64,
    rows: &[HistoryMass],
) -> Result<(), Error> {
    validate_rules(config)?;
    validate_denominator(denominator)?;
    if rows.len() != 32 {
        return Err(Error::InvalidReport(
            "history masses require exactly 32 rows",
        ));
    }
    let mut total = 0u64;
    let mut true_total = 0u64;
    for (index, row) in rows.iter().enumerate() {
        if row.observation.rules != *config || history_index(&row.observation) != index {
            return Err(Error::InvalidReport("history mass rules or order differ"));
        }
        if row.true_mass > row.total_mass {
            return Err(Error::InvalidReport("history true mass exceeds total mass"));
        }
        total = total
            .checked_add(row.total_mass)
            .ok_or(Error::ArithmeticOverflow)?;
        true_total = true_total
            .checked_add(row.true_mass)
            .ok_or(Error::ArithmeticOverflow)?;
    }
    if total != denominator || i128::from(true_total) * 2 != i128::from(denominator) {
        return Err(Error::InvalidReport(
            "history masses do not normalize with uniform live truth",
        ));
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct HistoryContribution {
    pub gain: i64,
    pub payoff: i64,
    pub utility: i64,
    pub optimal: i64,
    pub regret: i64,
}

/// Positive-mass history only; callers preserve absence on zero-mass rows.
pub(super) fn history_contribution(
    history: &HistoryMass,
    action: DecisionAction,
) -> Result<HistoryContribution, Error> {
    validate_rules(&history.observation.rules)?;
    if history.true_mass > history.total_mass {
        return Err(Error::InvalidReport("history true mass exceeds total mass"));
    }
    if history.total_mass == 0 {
        return Err(Error::ZeroEvidence);
    }
    let gain = i128::from(history.true_mass) * 2 - i128::from(history.total_mass);
    let gain = i64::try_from(gain).map_err(|_| Error::ArithmeticOverflow)?;
    let payoff = if action == DecisionAction::Intervene {
        gain
    } else {
        0
    };
    let utility = payoff.checked_neg().ok_or(Error::ArithmeticOverflow)?;
    let optimal = gain.max(0);
    let regret = optimal
        .checked_sub(payoff)
        .ok_or(Error::ArithmeticOverflow)?;
    Ok(HistoryContribution {
        gain,
        payoff,
        utility,
        optimal,
        regret,
    })
}

pub(super) fn score_histories(
    actions: &FrozenActions,
    denominator: u64,
    rows: &[HistoryMass],
) -> Result<Score, Error> {
    validate_history_masses(actions.config(), denominator, rows)?;
    let mut result = Score {
        rules: actions.config().clone(),
        denominator,
        payoff_numerator: 0,
        utility_numerator: 0,
    };
    for (history, row) in rows.iter().zip(actions.rows()) {
        if history.total_mass == 0 {
            continue;
        }
        let contribution = history_contribution(history, row.decision.action)?;
        result.payoff_numerator = result
            .payoff_numerator
            .checked_add(contribution.payoff)
            .ok_or(Error::ArithmeticOverflow)?;
        result.utility_numerator = result
            .utility_numerator
            .checked_add(contribution.utility)
            .ok_or(Error::ArithmeticOverflow)?;
    }
    result.validate()?;
    Ok(result)
}

pub(super) fn score_distribution(
    actions: &FrozenActions,
    distribution: &Distribution,
    policy: &Policy,
) -> Result<Score, Error> {
    Policy::new(policy.bits)?;
    if distribution.config() != actions.config() {
        return Err(Error::InvalidRules(
            "scoring distribution and action rules differ",
        ));
    }
    score_histories(
        actions,
        distribution.denominator(),
        &histories(distribution, policy)?,
    )
}

pub fn score(actions: &FrozenActions, policy: &Policy) -> Result<Score, Error> {
    Policy::new(policy.bits)?;
    score_distribution(actions, &enumerate(actions.config())?, policy)
}
