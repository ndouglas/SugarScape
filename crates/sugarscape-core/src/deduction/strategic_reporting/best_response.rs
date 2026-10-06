use super::{
    enumerate,
    enumeration::{checked_add, fixed_report, history_index, observation},
    Config, DecisionAction, Error, FrozenListener, Policy,
};
use serde::{Deserialize, Serialize};
/// Immutable equal-weight opponent panel, with all assumed-history actions frozen.
#[derive(Clone, Debug)]
pub struct TrainingPanel {
    config: Config,
    listeners: Vec<FrozenListener>,
    denominator: u64,
    contributions: [[[i64; 2]; 16]; 4],
}
impl TrainingPanel {
    pub fn new(config: &Config, listeners: Vec<FrozenListener>) -> Result<Self, Error> {
        config.validate()?;
        if listeners.is_empty() || listeners.len() > 16 {
            return Err(Error::InvalidPanel(
                "requires one to sixteen equally weighted listeners".into(),
            ));
        }
        let distribution = enumerate(config)?;
        let denominator = distribution
            .denominator()
            .checked_mul(listeners.len() as u64)
            .ok_or(Error::ArithmeticOverflow)?;
        let mut actions = Vec::with_capacity(listeners.len());
        for (slot, listener) in listeners.iter().enumerate() {
            let mut table = [DecisionAction::Abstain; 32];
            for (index, action) in table.iter_mut().enumerate() {
                *action = listener
                    .decide(&observation(config, index))
                    .map_err(|error| {
                        Error::InvalidPanel(format!(
                            "listener {slot} rejects assumed public history {index}: {error}"
                        ))
                    })?;
            }
            actions.push(table);
        }
        let mut contributions = [[[0i64; 2]; 16]; 4];
        for (calibration, rows) in contributions.iter_mut().enumerate() {
            let policy = Policy::new(calibration as u32)?;
            for world in &distribution.worlds {
                let own_calibration = policy.calibration(world.calibration_signals[0]);
                let calibration_reports = [
                    own_calibration,
                    fixed_report(world.fixed_profile, world.calibration_signals[1]),
                ];
                let row = (usize::from(world.calibration_signals[0]) << 3)
                    | (usize::from(own_calibration) << 2)
                    | (usize::from(world.calibration_truth) << 1)
                    | usize::from(world.live_signals[0]);
                for (report, utility) in rows[row].iter_mut().enumerate() {
                    let index = history_index(
                        world.calibration_truth,
                        calibration_reports,
                        [
                            report != 0,
                            fixed_report(world.fixed_profile, world.live_signals[1]),
                        ],
                    );
                    for table in &actions {
                        let value = i64::try_from(world.mass)
                            .map_err(|_| Error::ArithmeticOverflow)?
                            .checked_mul(i64::from(
                                config
                                    .strategic_utility
                                    .utility(world.live_truth, table[index]),
                            ))
                            .ok_or(Error::ArithmeticOverflow)?;
                        checked_add(utility, value)?;
                    }
                }
            }
        }
        Ok(Self {
            config: config.clone(),
            listeners,
            denominator,
            contributions,
        })
    }
    pub fn config(&self) -> &Config {
        &self.config
    }
    pub fn listeners(&self) -> &[FrozenListener] {
        &self.listeners
    }
    pub fn denominator(&self) -> u64 {
        self.denominator
    }
    pub fn fitness(&self, policy: &Policy) -> Result<i64, Error> {
        Policy::new(policy.bits)?;
        let mut score = 0;
        for (row, values) in self.contributions[(policy.bits & 3) as usize]
            .iter()
            .enumerate()
        {
            checked_add(
                &mut score,
                values[((policy.bits >> (2 + row)) & 1) as usize],
            )?;
        }
        Ok(score)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BestResponse {
    pub policy: Policy,
    pub fitness_numerator: i64,
    pub denominator: u64,
}
pub fn exact_best_response(panel: &TrainingPanel) -> Result<BestResponse, Error> {
    let mut best: Option<BestResponse> = None;
    for (calibration, rows) in panel.contributions.iter().enumerate() {
        let mut bits = calibration as u32;
        for (row, values) in rows.iter().enumerate() {
            // False on equality includes every unreachable row (both contributions zero).
            if values[1] > values[0] {
                bits |= 1 << (2 + row);
            }
        }
        let policy = Policy::new(bits)?;
        let fitness_numerator = panel.fitness(&policy)?;
        let candidate = BestResponse {
            policy,
            fitness_numerator,
            denominator: panel.denominator,
        };
        if best.as_ref().is_none_or(|previous| {
            candidate.fitness_numerator > previous.fitness_numerator
                || (candidate.fitness_numerator == previous.fitness_numerator
                    && candidate.policy.bits < previous.policy.bits)
        }) {
            best = Some(candidate);
        }
    }
    best.ok_or_else(|| Error::InvalidPanel("no calibration tables".into()))
}
