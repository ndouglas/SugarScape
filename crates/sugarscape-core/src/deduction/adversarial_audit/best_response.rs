use super::{scoring::score_distribution, Error, FrozenActions, Score};
use crate::deduction::strategic_reporting::{enumerate, Config, Policy};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BasisRow {
    pub calibration: u8,
    pub base_utility_numerator: i64,
    /// None means structurally unreachable, including when other rows have zero deltas.
    pub deltas: [Option<i64>; 16],
}

/// Immutable 36-query decomposition using complete public history masses.
#[derive(Clone, Debug)]
pub struct AttackBasis {
    config: Config,
    denominator: u64,
    rows: [BasisRow; 4],
}

fn reachable(policy: &Policy, row: usize) -> bool {
    (row & 4 != 0) == policy.calibration(row & 8 != 0)
}

impl AttackBasis {
    pub fn build(actions: &FrozenActions) -> Result<Self, Error> {
        let distribution = enumerate(actions.config())?;
        let mut rows = Vec::with_capacity(4);
        for calibration in 0..4u8 {
            let base = Policy::new(u32::from(calibration))?;
            let base_utility_numerator =
                score_distribution(actions, &distribution, &base)?.utility_numerator;
            let mut deltas = [None; 16];
            for (row, delta) in deltas.iter_mut().enumerate() {
                if reachable(&base, row) {
                    let flipped = Policy::new(base.bits | 1 << (2 + row))?;
                    *delta = Some(
                        score_distribution(actions, &distribution, &flipped)?
                            .utility_numerator
                            .checked_sub(base_utility_numerator)
                            .ok_or(Error::ArithmeticOverflow)?,
                    );
                }
            }
            rows.push(BasisRow {
                calibration,
                base_utility_numerator,
                deltas,
            });
        }
        Ok(Self {
            config: actions.config().clone(),
            denominator: distribution.denominator(),
            rows: rows
                .try_into()
                .map_err(|_| Error::InvalidReport("basis requires four calibration tables"))?,
        })
    }
    pub fn denominator(&self) -> u64 {
        self.denominator
    }
    pub fn rows(&self) -> &[BasisRow; 4] {
        &self.rows
    }
    pub fn fitness(&self, policy: &Policy) -> Result<i64, Error> {
        Policy::new(policy.bits)?;
        let basis = &self.rows[(policy.bits & 3) as usize];
        let mut utility = basis.base_utility_numerator;
        for (row, delta) in basis.deltas.iter().enumerate() {
            if policy.bits & (1 << (2 + row)) != 0 {
                if let Some(delta) = delta {
                    utility = utility
                        .checked_add(*delta)
                        .ok_or(Error::ArithmeticOverflow)?;
                }
            }
        }
        Ok(utility)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FitnessRow {
    pub canonical_bits: u32,
    pub utility_numerator: i64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BestResponse {
    pub policy: Policy,
    pub score: Score,
}

/// Four calibration tables, each assigning its eight structurally reachable live rows.
pub fn canonical_policies() -> Vec<Policy> {
    let mut policies = Vec::with_capacity(1024);
    for calibration in 0..4u32 {
        let base = Policy { bits: calibration };
        let rows: Vec<_> = (0..16).filter(|row| reachable(&base, *row)).collect();
        for assignment in 0..256u32 {
            let mut bits = calibration;
            for (position, row) in rows.iter().enumerate() {
                if assignment & (1 << position) != 0 {
                    bits |= 1 << (2 + row);
                }
            }
            policies.push(Policy { bits });
        }
    }
    policies.sort_unstable_by_key(|policy| policy.bits);
    policies
}

pub fn exact_best_response(basis: &AttackBasis) -> Result<BestResponse, Error> {
    let mut best: Option<BestResponse> = None;
    for row in basis.rows() {
        let mut bits = u32::from(row.calibration);
        for (live_row, delta) in row.deltas.iter().enumerate() {
            // False on zero gives the smallest encoding among row-wise ties.
            if delta.is_some_and(|delta| delta > 0) {
                bits |= 1 << (2 + live_row);
            }
        }
        let policy = Policy::new(bits)?;
        let utility_numerator = basis.fitness(&policy)?;
        let score = Score {
            rules: basis.config.clone(),
            denominator: basis.denominator,
            payoff_numerator: utility_numerator
                .checked_neg()
                .ok_or(Error::ArithmeticOverflow)?,
            utility_numerator,
        };
        score.validate()?;
        let candidate = BestResponse { policy, score };
        if best.as_ref().is_none_or(|previous| {
            candidate.score.utility_numerator > previous.score.utility_numerator
                || (candidate.score.utility_numerator == previous.score.utility_numerator
                    && candidate.policy.bits < previous.policy.bits)
        }) {
            best = Some(candidate);
        }
    }
    best.ok_or(Error::InvalidReport("basis has no calibration candidates"))
}

pub fn fitness_table(basis: &AttackBasis) -> Result<Vec<FitnessRow>, Error> {
    canonical_policies()
        .into_iter()
        .map(|policy| {
            Ok(FitnessRow {
                canonical_bits: policy.bits,
                utility_numerator: basis.fitness(&policy)?,
            })
        })
        .collect()
}
