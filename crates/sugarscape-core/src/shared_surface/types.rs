use serde::{Deserialize, Deserializer, Serialize};
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Mechanism {
    SharedPersistent,
    SharedResetting,
    PrivatePersistent,
    Inert,
}

impl Mechanism {
    pub const ALL: [Self; 4] = [
        Self::SharedPersistent,
        Self::SharedResetting,
        Self::PrivatePersistent,
        Self::Inert,
    ];

    pub const fn index(self) -> usize {
        match self {
            Self::SharedPersistent => 0,
            Self::SharedResetting => 1,
            Self::PrivatePersistent => 2,
            Self::Inert => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Environment {
    InFamily(Mechanism),
    DataFlip,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Role {
    A,
    B,
}

impl Role {
    pub const fn index(self) -> usize {
        match self {
            Self::A => 0,
            Self::B => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Symbol {
    Blank,
    Probe0,
    Ack0,
    Probe1,
    Ack1,
    Data0,
    Data1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Action {
    Read,
    Write(Symbol),
    Wait,
    InspectOwnTarget,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Outcome {
    Read(Symbol),
    Accepted,
    Waited,
    Inspected(bool),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum Phase {
    Calibration,
    Live { trial: u8 },
}

impl Phase {
    pub fn validate(self) -> Result<(), Error> {
        match self {
            Self::Live { trial } if trial >= super::TRIALS => {
                Err(Error::InvalidProtocol("live trial must be in 0..4".into()))
            }
            _ => Ok(()),
        }
    }
}

impl<'de> Deserialize<'de> for Phase {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        enum Wire {
            Calibration,
            Live { trial: u8 },
        }
        let phase = match Wire::deserialize(deserializer)? {
            Wire::Calibration => Self::Calibration,
            Wire::Live { trial } => Self::Live { trial },
        };
        phase.validate().map_err(serde::de::Error::custom)?;
        Ok(phase)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Position {
    pub phase: Phase,
    pub round: u8,
    pub slot: u8,
}

impl Position {
    pub fn validate(self) -> Result<(), Error> {
        self.phase.validate()?;
        if !(1..=3).contains(&self.round) || !(1..=4).contains(&self.slot) {
            return Err(Error::InvalidProtocol(
                "position requires round 1..=3 and slot 1..=4".into(),
            ));
        }
        Ok(())
    }
}

impl<'de> Deserialize<'de> for Position {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            phase: Phase,
            round: u8,
            slot: u8,
        }
        let wire = Wire::deserialize(deserializer)?;
        let position = Self {
            phase: wire.phase,
            round: wire.round,
            slot: wire.slot,
        };
        position.validate().map_err(serde::de::Error::custom)?;
        Ok(position)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum Checkpoint {
    EpisodeStart,
    Action(Position),
    /// After the paid outcome and before a round/phase reset or future private bit.
    AfterSlot(Position),
    Prediction {
        trial: u8,
    },
}

impl<'de> Deserialize<'de> for Checkpoint {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        enum Wire {
            EpisodeStart,
            Action(Position),
            AfterSlot(Position),
            Prediction { trial: u8 },
        }
        match Wire::deserialize(deserializer)? {
            Wire::EpisodeStart => Ok(Self::EpisodeStart),
            Wire::Action(position) => Ok(Self::Action(position)),
            Wire::AfterSlot(position) => Ok(Self::AfterSlot(position)),
            Wire::Prediction { trial } => {
                Phase::Live { trial }
                    .validate()
                    .map_err(serde::de::Error::custom)?;
                Ok(Self::Prediction { trial })
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum OwnPrior {
    Uniform,
    PointMass(Mechanism),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ids {
    pub surface: String,
    pub agents: [String; 2],
}

impl Ids {
    pub fn validate(&self) -> Result<(), Error> {
        if self.surface.trim().is_empty() || self.agents.iter().any(|id| id.trim().is_empty()) {
            return Err(Error::InvalidProtocol(
                "surface and Agent identifiers must be nonblank".into(),
            ));
        }
        if self.agents[0] == self.agents[1] {
            return Err(Error::InvalidProtocol(
                "Agent identifiers must be distinct".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub position: Position,
    pub action: Action,
    pub outcome: Outcome,
    pub credits_after: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum LocalEntry {
    Reset { phase: Phase },
    PrivateBit { trial: u8, bit: bool },
    Action(Event),
}

impl<'de> Deserialize<'de> for LocalEntry {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        enum Wire {
            Reset { phase: Phase },
            PrivateBit { trial: u8, bit: bool },
            Action(Event),
        }
        match Wire::deserialize(deserializer)? {
            Wire::Reset { phase } => Ok(Self::Reset { phase }),
            Wire::Action(event) => Ok(Self::Action(event)),
            Wire::PrivateBit { trial, bit } => {
                Phase::Live { trial }
                    .validate()
                    .map_err(serde::de::Error::custom)?;
                Ok(Self::PrivateBit { trial, bit })
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocalPrefix {
    pub role: Role,
    pub ids: Ids,
    pub checkpoint: Checkpoint,
    pub own_prior: OwnPrior,
    pub entries: Vec<LocalEntry>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Knowledge {
    Unknown,
    Known,
    NoCommunication,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pair {
    pub roles: [Knowledge; 2],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PriorMode {
    Treatment,
    RestartUniform,
    Stale(Mechanism),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Protocol {
    pub calibration_rounds: u8,
    pub pair: Pair,
    pub prior_mode: PriorMode,
    pub ids: Ids,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EpisodeBits(pub [(bool, bool); 4]);

impl EpisodeBits {
    pub fn from_index(index: u16) -> Result<Self, Error> {
        if index >= 256 {
            return Err(Error::InvalidProtocol(
                "episode bit index must be in 0..256".into(),
            ));
        }
        Ok(Self(std::array::from_fn(|trial| {
            (
                index & (1 << (2 * trial)) != 0,
                index & (1 << (2 * trial + 1)) != 0,
            )
        })))
    }

    pub fn index(&self) -> u16 {
        self.0
            .iter()
            .enumerate()
            .fold(0, |index, (trial, &(x, y))| {
                index | (u16::from(x) << (2 * trial)) | (u16::from(y) << (2 * trial + 1))
            })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Probability {
    pub numerator: u64,
    pub denominator: u64,
}

impl Probability {
    pub fn new(numerator: u64, denominator: u64) -> Result<Self, Error> {
        if denominator == 0 || numerator > denominator {
            return Err(Error::InvalidProtocol(
                "probability requires 0 <= numerator <= positive denominator".into(),
            ));
        }
        let divisor = gcd(numerator, denominator);
        Ok(Self {
            numerator: numerator / divisor,
            denominator: denominator / divisor,
        })
    }

    pub fn checked_add(self, other: Self) -> Result<Self, Error> {
        Self::new(self.numerator, self.denominator)?;
        Self::new(other.numerator, other.denominator)?;
        let numerator = self
            .numerator
            .checked_mul(other.denominator)
            .and_then(|a| {
                other
                    .numerator
                    .checked_mul(self.denominator)
                    .and_then(|b| a.checked_add(b))
            })
            .ok_or(Error::ArithmeticOverflow)?;
        let denominator = self
            .denominator
            .checked_mul(other.denominator)
            .ok_or(Error::ArithmeticOverflow)?;
        Self::new(numerator, denominator)
    }

    pub fn checked_mul(self, other: Self) -> Result<Self, Error> {
        Self::new(self.numerator, self.denominator)?;
        Self::new(other.numerator, other.denominator)?;
        Self::new(
            self.numerator
                .checked_mul(other.numerator)
                .ok_or(Error::ArithmeticOverflow)?,
            self.denominator
                .checked_mul(other.denominator)
                .ok_or(Error::ArithmeticOverflow)?,
        )
    }

    pub fn checked_cmp(self, other: Self) -> Result<Ordering, Error> {
        Self::new(self.numerator, self.denominator)?;
        Self::new(other.numerator, other.denominator)?;
        Ok(self
            .numerator
            .checked_mul(other.denominator)
            .ok_or(Error::ArithmeticOverflow)?
            .cmp(
                &other
                    .numerator
                    .checked_mul(self.denominator)
                    .ok_or(Error::ArithmeticOverflow)?,
            ))
    }
}

impl<'de> Deserialize<'de> for Probability {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            numerator: u64,
            denominator: u64,
        }
        let wire = Wire::deserialize(deserializer)?;
        Self::new(wire.numerator, wire.denominator).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct ScoreFraction {
    pub numerator: i64,
    pub denominator: u64,
}

impl ScoreFraction {
    pub fn new(numerator: i64, denominator: u64) -> Result<Self, Error> {
        if denominator == 0 {
            return Err(Error::InvalidProtocol(
                "score fraction requires a positive denominator".into(),
            ));
        }
        let divisor = gcd(numerator.unsigned_abs(), denominator);
        Ok(Self {
            numerator: (i128::from(numerator) / i128::from(divisor)) as i64,
            denominator: denominator / divisor,
        })
    }

    pub fn checked_add(self, other: Self) -> Result<Self, Error> {
        Self::new(self.numerator, self.denominator)?;
        Self::new(other.numerator, other.denominator)?;
        let numerator = i128::from(self.numerator)
            .checked_mul(i128::from(other.denominator))
            .and_then(|a| {
                i128::from(other.numerator)
                    .checked_mul(i128::from(self.denominator))
                    .and_then(|b| a.checked_add(b))
            })
            .ok_or(Error::ArithmeticOverflow)?;
        let denominator = self
            .denominator
            .checked_mul(other.denominator)
            .ok_or(Error::ArithmeticOverflow)?;
        Self::from_wide(numerator, denominator)
    }

    pub fn checked_mul(self, other: Self) -> Result<Self, Error> {
        Self::new(self.numerator, self.denominator)?;
        Self::new(other.numerator, other.denominator)?;
        let numerator = i128::from(self.numerator)
            .checked_mul(i128::from(other.numerator))
            .ok_or(Error::ArithmeticOverflow)?;
        let denominator = self
            .denominator
            .checked_mul(other.denominator)
            .ok_or(Error::ArithmeticOverflow)?;
        Self::from_wide(numerator, denominator)
    }

    pub fn checked_cmp(self, other: Self) -> Result<Ordering, Error> {
        Self::new(self.numerator, self.denominator)?;
        Self::new(other.numerator, other.denominator)?;
        Ok(i128::from(self.numerator)
            .checked_mul(i128::from(other.denominator))
            .ok_or(Error::ArithmeticOverflow)?
            .cmp(
                &i128::from(other.numerator)
                    .checked_mul(i128::from(self.denominator))
                    .ok_or(Error::ArithmeticOverflow)?,
            ))
    }

    fn from_wide(numerator: i128, denominator: u64) -> Result<Self, Error> {
        let divisor = gcd_wide(numerator.unsigned_abs(), u128::from(denominator));
        let numerator =
            numerator / i128::try_from(divisor).map_err(|_| Error::ArithmeticOverflow)?;
        Self::new(
            i64::try_from(numerator).map_err(|_| Error::ArithmeticOverflow)?,
            denominator / divisor as u64,
        )
    }
}

impl<'de> Deserialize<'de> for ScoreFraction {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            numerator: i64,
            denominator: u64,
        }
        let wire = Wire::deserialize(deserializer)?;
        Self::new(wire.numerator, wire.denominator).map_err(serde::de::Error::custom)
    }
}

fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let remainder = a % b;
        a = b;
        b = remainder;
    }
    a
}

fn gcd_wide(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        let remainder = a % b;
        a = b;
        b = remainder;
    }
    a
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Belief {
    pub models: [Probability; 4],
    pub target: Option<Probability>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocalView {
    pub prefix: LocalPrefix,
    pub credits: u8,
    pub private_bit: Option<bool>,
    pub belief: Belief,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Decision {
    pub action: Action,
}

/// Evaluator-only write origin. It is never part of a local controller observation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Lineage {
    pub writer: Role,
    pub write_position: Position,
    pub symbol: Symbol,
    pub task_trial: Option<u8>,
}

impl<'de> Deserialize<'de> for Lineage {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            writer: Role,
            write_position: Position,
            symbol: Symbol,
            task_trial: Option<u8>,
        }
        let wire = Wire::deserialize(deserializer)?;
        if let Some(trial) = wire.task_trial {
            Phase::Live { trial }
                .validate()
                .map_err(serde::de::Error::custom)?;
        }
        Ok(Self {
            writer: wire.writer,
            write_position: wire.write_position,
            symbol: wire.symbol,
            task_trial: wire.task_trial,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Error {
    InvalidProtocol(String),
    InvalidAction(String),
    InvalidReport(String),
    UnsupportedHistory { prefix: LocalPrefix },
    ArithmeticOverflow,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidProtocol(message) => {
                write!(f, "invalid shared-surface protocol: {message}")
            }
            Self::InvalidAction(message) => write!(f, "invalid shared-surface action: {message}"),
            Self::InvalidReport(message) => write!(f, "invalid shared-surface report: {message}"),
            Self::UnsupportedHistory { prefix } => write!(
                f,
                "unsupported shared-surface history for Agent {:?} at {:?}",
                prefix.role, prefix.checkpoint
            ),
            Self::ArithmeticOverflow => f.write_str("shared-surface exact arithmetic overflow"),
        }
    }
}

impl std::error::Error for Error {}
