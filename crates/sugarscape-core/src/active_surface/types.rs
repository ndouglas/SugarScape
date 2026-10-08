use serde::{Deserialize, Deserializer, Serialize};

use crate::shared_surface::{action_cost, role_at, validate_action, Outcome};
pub use crate::shared_surface::{
    Action, Belief, Environment, EpisodeBits, Event, Ids, LocalEntry, Mechanism, OwnPrior, Phase,
    Position, Probability, Role, ScoreFraction, Symbol,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PolicyKind {
    Adaptive,
    FixedThree,
    NoProbe,
    InspectOnly,
    Known,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Choice {
    ContinueProbe,
    StopProbing,
    Inspect,
    AttemptCommunication,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum Boundary {
    ProbeChoice { completed: u8 },
    TrialChoice { trial: u8 },
}

impl Boundary {
    pub fn validate(self) -> Result<(), Error> {
        match self {
            Self::ProbeChoice { completed } if completed > 3 => Err(Error::InvalidProtocol(
                "completed probes must be in 0..=3".into(),
            )),
            Self::TrialChoice { trial } if trial > 3 => {
                Err(Error::InvalidProtocol("trial must be in 0..=3".into()))
            }
            _ => Ok(()),
        }
    }
    pub fn validate_choice(self, choice: Choice) -> Result<(), Error> {
        self.validate()?;
        match (self, choice) {
            (Self::ProbeChoice { completed: 0..=2 }, Choice::ContinueProbe)
            | (Self::ProbeChoice { .. }, Choice::StopProbing)
            | (Self::TrialChoice { .. }, Choice::Inspect | Choice::AttemptCommunication) => Ok(()),
            _ => invalid("choice is not legal at this boundary"),
        }
    }
    pub fn trial_position(self, role: Role) -> Result<Position, Error> {
        self.validate()?;
        let Self::TrialChoice { trial } = self else {
            return invalid("probe boundary has no trial position");
        };
        Ok(Position {
            phase: Phase::Live { trial },
            round: 1,
            slot: if role == Role::A { 1 } else { 2 },
        })
    }
}
impl<'de> Deserialize<'de> for Boundary {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        enum Wire {
            ProbeChoice { completed: u8 },
            TrialChoice { trial: u8 },
        }
        let value = match Wire::deserialize(d)? {
            Wire::ProbeChoice { completed } => Self::ProbeChoice { completed },
            Wire::TrialChoice { trial } => Self::TrialChoice { trial },
        };
        value.validate().map_err(serde::de::Error::custom)?;
        Ok(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum Checkpoint {
    Boundary(Boundary),
    BeforeSlot(Position),
    AfterSlot(Position),
    Prediction { trial: u8 },
    Finished,
}
impl Checkpoint {
    pub fn validate(self) -> Result<(), Error> {
        match self {
            Self::Boundary(b) => b.validate(),
            Self::BeforeSlot(p) | Self::AfterSlot(p) => p.validate().map_err(Error::from),
            Self::Prediction { trial } => Boundary::TrialChoice { trial }.validate(),
            Self::Finished => Ok(()),
        }
    }
}
impl<'de> Deserialize<'de> for Checkpoint {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        enum Wire {
            Boundary(Boundary),
            BeforeSlot(Position),
            AfterSlot(Position),
            Prediction { trial: u8 },
            Finished,
        }
        let value = match Wire::deserialize(d)? {
            Wire::Boundary(b) => Self::Boundary(b),
            Wire::BeforeSlot(p) => Self::BeforeSlot(p),
            Wire::AfterSlot(p) => Self::AfterSlot(p),
            Wire::Prediction { trial } => Self::Prediction { trial },
            Wire::Finished => Self::Finished,
        };
        value.validate().map_err(serde::de::Error::custom)?;
        Ok(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum Entry {
    Physical(LocalEntry),
    OwnChoice { boundary: Boundary, choice: Choice },
    PublicProbeStop { completed: u8 },
}
impl<'de> Deserialize<'de> for Entry {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        enum Wire {
            Physical(LocalEntry),
            OwnChoice { boundary: Boundary, choice: Choice },
            PublicProbeStop { completed: u8 },
        }
        match Wire::deserialize(d)? {
            Wire::Physical(e) => Ok(Self::Physical(e)),
            Wire::OwnChoice { boundary, choice } => {
                boundary
                    .validate_choice(choice)
                    .map_err(serde::de::Error::custom)?;
                Ok(Self::OwnChoice { boundary, choice })
            }
            Wire::PublicProbeStop { completed } => {
                Boundary::ProbeChoice { completed }
                    .validate()
                    .map_err(serde::de::Error::custom)?;
                Ok(Self::PublicProbeStop { completed })
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Protocol {
    pub experimenter: Role,
    pub policy: PolicyKind,
    pub ids: Ids,
}
impl Protocol {
    pub fn new(experimenter: Role, policy: PolicyKind) -> Self {
        Self {
            experimenter,
            policy,
            ids: Ids {
                surface: "status-field".into(),
                agents: ["Agent-A".into(), "Agent-B".into()],
            },
        }
    }
    pub fn validate(&self) -> Result<(), Error> {
        self.ids
            .validate()
            .map_err(|e| Error::InvalidProtocol(e.to_string()))
    }
    /// Enforce the declared choice owner, opaque IDs, and supplied prior kind.
    pub fn validate_prefix(&self, prefix: &Prefix) -> Result<(), Error> {
        self.validate()?;
        prefix.validate()?;
        if prefix.ids != self.ids {
            return invalid("prefix identifiers differ from protocol");
        }
        if prefix.role != self.experimenter
            && prefix
                .entries
                .iter()
                .any(|e| matches!(e, Entry::OwnChoice { .. }))
        {
            return invalid("responder cannot receive the experimenter's private choices");
        }
        let valid_prior = if prefix.role == self.experimenter && self.policy != PolicyKind::Known {
            prefix.own_prior == OwnPrior::Uniform
        } else {
            matches!(prefix.own_prior, OwnPrior::PointMass(_))
        };
        if !valid_prior {
            return invalid("own prior disagrees with declared policy role");
        }
        if prefix.role == self.experimenter {
            for entry in &prefix.entries {
                let required = match entry {
                    Entry::PublicProbeStop { completed } => Some((
                        Boundary::ProbeChoice {
                            completed: *completed,
                        },
                        Choice::StopProbing,
                    )),
                    Entry::Physical(LocalEntry::Action(event))
                        if event.position.phase == Phase::Calibration =>
                    {
                        Some((
                            Boundary::ProbeChoice {
                                completed: event.position.round - 1,
                            },
                            Choice::ContinueProbe,
                        ))
                    }
                    _ => None,
                };
                if let Some((boundary, choice)) = required {
                    if !prefix
                        .entries
                        .contains(&Entry::OwnChoice { boundary, choice })
                    {
                        return invalid("experimenter history omitted its own probe intervention");
                    }
                }
                if let Entry::Physical(LocalEntry::Action(Event {
                    position:
                        Position {
                            phase: Phase::Live { trial },
                            ..
                        },
                    ..
                })) = entry
                {
                    if !prefix.entries.iter().any(|e| matches!(e, Entry::OwnChoice { boundary: Boundary::TrialChoice { trial: chosen }, .. } if chosen == trial)) { return invalid("experimenter history omitted its own live routine"); }
                }
            }
        }
        Ok(())
    }
}
impl<'de> Deserialize<'de> for Protocol {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            experimenter: Role,
            policy: PolicyKind,
            ids: Ids,
        }
        let w = Wire::deserialize(d)?;
        let p = Self {
            experimenter: w.experimenter,
            policy: w.policy,
            ids: w.ids,
        };
        p.validate().map_err(serde::de::Error::custom)?;
        Ok(p)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Prefix {
    pub role: Role,
    pub ids: Ids,
    pub own_prior: OwnPrior,
    pub checkpoint: Checkpoint,
    pub entries: Vec<Entry>,
}
impl Prefix {
    pub fn validate(&self) -> Result<(), Error> {
        self.local_state().map(|_| ())
    }

    /// Validate chronology and derive local credits/bits from own entries alone.
    fn local_state(&self) -> Result<(u8, Option<bool>), Error> {
        self.ids.validate()?;
        self.checkpoint.validate()?;
        let mut phase = None;
        let mut actions = 0u8;
        let mut calibration_actions = 0u8;
        let mut credits = 48u8;
        let mut bit = None;
        let mut needs_bit = false;
        let mut stop = None;
        let mut probe_choice = None;
        let mut trial_choice = None;
        for entry in &self.entries {
            if needs_bit && !matches!(entry, Entry::Physical(LocalEntry::PrivateBit { .. })) {
                return invalid("live reset must immediately issue the own bit");
            }
            match entry {
                Entry::OwnChoice { boundary, choice } => {
                    boundary.validate_choice(*choice)?;
                    match *boundary {
                        Boundary::ProbeChoice { completed } => {
                            if stop.is_some()
                                || matches!(phase, Some(Phase::Live { .. }))
                                || calibration_actions != 2 * completed
                                || probe_choice.is_some_and(|(last, _)| last == completed)
                            {
                                return invalid("duplicate or mistimed probe choice");
                            }
                            probe_choice = Some((completed, *choice));
                        }
                        Boundary::TrialChoice { trial } => {
                            if phase != Some(Phase::Live { trial })
                                || bit.is_none()
                                || actions != 0
                                || trial_choice == Some(trial)
                            {
                                return invalid("duplicate or mistimed trial choice");
                            }
                            trial_choice = Some(trial);
                        }
                    }
                }
                Entry::PublicProbeStop { completed } => {
                    Boundary::ProbeChoice {
                        completed: *completed,
                    }
                    .validate()?;
                    if stop.is_some()
                        || matches!(phase, Some(Phase::Live { .. }))
                        || calibration_actions != 2 * completed
                        || (*completed == 0 && phase.is_some())
                        || probe_choice.is_some_and(|c| c != (*completed, Choice::StopProbing))
                    {
                        return invalid("duplicate or mistimed public stop");
                    }
                    stop = Some(*completed);
                }
                Entry::Physical(LocalEntry::Reset { phase: next }) => {
                    next.validate()?;
                    match *next {
                        Phase::Calibration => {
                            if phase.is_some()
                                || stop.is_some()
                                || probe_choice.is_some_and(|c| c != (0, Choice::ContinueProbe))
                            {
                                return invalid("calibration reset requires an actual first probe");
                            }
                        }
                        Phase::Live { trial: 0 } => {
                            if stop.is_none() || matches!(phase, Some(Phase::Live { .. })) {
                                return invalid("live phase requires public probing stop");
                            }
                        }
                        Phase::Live { trial } => {
                            if phase != Some(Phase::Live { trial: trial - 1 }) || actions != 6 {
                                return invalid("live reset skips an unfinished trial");
                            }
                        }
                    }
                    phase = Some(*next);
                    actions = 0;
                    bit = None;
                    needs_bit = matches!(next, Phase::Live { .. });
                }
                Entry::Physical(LocalEntry::PrivateBit { trial, bit: issued }) => {
                    if !needs_bit || phase != Some(Phase::Live { trial: *trial }) || bit.is_some() {
                        return invalid("duplicate, missing, or future own bit");
                    }
                    bit = Some(*issued);
                    needs_bit = false;
                }
                Entry::Physical(LocalEntry::Action(event)) => {
                    if phase != Some(event.position.phase)
                        || (event.position.phase == Phase::Calibration && stop.is_some())
                    {
                        return invalid("paid action outside current phase");
                    }
                    let slots = if self.role == Role::A { [1, 4] } else { [2, 3] };
                    if actions >= 6
                        || event.position.round != actions / 2 + 1
                        || event.position.slot != slots[usize::from(actions % 2)]
                    {
                        return invalid("missing, repeated, or out-of-order own opportunity");
                    }
                    if event.position.phase == Phase::Calibration {
                        if probe_choice
                            .is_some_and(|c| c != (event.position.round - 1, Choice::ContinueProbe))
                        {
                            return invalid("probe action lacks its continue choice");
                        }
                        calibration_actions += 1;
                    } else if bit.is_none()
                        || (probe_choice.is_some()
                            && !matches!(event.position.phase, Phase::Live { trial } if trial_choice == Some(trial)))
                    {
                        return invalid("live action lacks issued bit or own selected routine");
                    }
                    validate_action(self.role, event.position, &event.action)?;
                    credits = credits
                        .checked_sub(action_cost(&event.action))
                        .ok_or_else(|| {
                            Error::InvalidHistory("insufficient local credits".into())
                        })?;
                    if credits != event.credits_after {
                        return invalid("paid outcome credits disagree with action cost");
                    }
                    if !matches!(
                        (event.action, event.outcome),
                        (Action::Read, Outcome::Read(_))
                            | (Action::Write(_), Outcome::Accepted)
                            | (Action::Wait, Outcome::Waited)
                            | (Action::InspectOwnTarget, Outcome::Inspected(_))
                    ) {
                        return invalid("outcome does not match paid action");
                    }
                    actions += 1;
                }
            }
        }
        if needs_bit {
            return invalid("live phase omitted its own bit");
        }
        match self.checkpoint {
            Checkpoint::Boundary(Boundary::ProbeChoice { completed }) => {
                if matches!(phase, Some(Phase::Live { .. }))
                    || calibration_actions != 2 * completed
                    || (completed == 0 && phase.is_some())
                {
                    return invalid("probe checkpoint disagrees with completed rounds");
                }
            }
            Checkpoint::Boundary(Boundary::TrialChoice { trial }) => {
                if phase != Some(Phase::Live { trial }) || actions != 0 {
                    return invalid("trial boundary is not first owned opportunity");
                }
            }
            Checkpoint::BeforeSlot(position) | Checkpoint::AfterSlot(position) => {
                let inclusive = matches!(self.checkpoint, Checkpoint::AfterSlot(_));
                let expected = (1..=3)
                    .flat_map(|round| {
                        (1..=4).map(move |slot| Position {
                            phase: position.phase,
                            round,
                            slot,
                        })
                    })
                    .filter(|p| {
                        (p.round, p.slot) < (position.round, position.slot)
                            || (inclusive && *p == position)
                    })
                    .filter(|p| role_at(*p).ok() == Some(self.role))
                    .count();
                if phase != Some(position.phase)
                    || usize::from(actions) != expected
                    || (position.phase == Phase::Calibration && stop.is_some())
                {
                    return invalid("paid checkpoint disagrees with local chronology");
                }
            }
            Checkpoint::Prediction { trial } => {
                if phase != Some(Phase::Live { trial }) || actions != 6 {
                    return invalid("prediction precedes trial completion");
                }
            }
            Checkpoint::Finished => {
                if phase != Some(Phase::Live { trial: 3 }) || actions != 6 {
                    return invalid("finished checkpoint precedes four trials");
                }
            }
        }
        Ok((credits, bit))
    }
}
impl<'de> Deserialize<'de> for Prefix {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            role: Role,
            ids: Ids,
            own_prior: OwnPrior,
            checkpoint: Checkpoint,
            entries: Vec<Entry>,
        }
        let w = Wire::deserialize(d)?;
        let p = Self {
            role: w.role,
            ids: w.ids,
            own_prior: w.own_prior,
            checkpoint: w.checkpoint,
            entries: w.entries,
        };
        p.validate().map_err(serde::de::Error::custom)?;
        Ok(p)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct View {
    pub prefix: Prefix,
    pub credits: u8,
    pub private_bit: Option<bool>,
    pub belief: Belief,
}
impl View {
    pub fn from_prefix(prefix: Prefix, belief: Belief) -> Result<Self, Error> {
        let (credits, private_bit) = prefix.local_state()?;
        let total = belief
            .models
            .iter()
            .try_fold(Probability::new(0, 1)?, |sum, p| sum.checked_add(*p))?;
        if total != Probability::new(1, 1)? {
            return invalid("model probabilities must sum to one");
        }
        if let Some(p) = belief.target {
            Probability::new(p.numerator, p.denominator)?;
        }
        if private_bit.is_none() && belief.target.is_some() {
            return invalid("target posterior requires an issued live task bit");
        }
        Ok(Self {
            prefix,
            credits,
            private_bit,
            belief,
        })
    }
}
impl<'de> Deserialize<'de> for View {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            prefix: Prefix,
            credits: u8,
            private_bit: Option<bool>,
            belief: Belief,
        }
        let w = Wire::deserialize(d)?;
        let view = Self::from_prefix(w.prefix, w.belief).map_err(serde::de::Error::custom)?;
        if view.credits != w.credits || view.private_bit != w.private_bit {
            return Err(serde::de::Error::custom(
                "view credits/private bit disagree with own entries",
            ));
        }
        Ok(view)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Error {
    InvalidProtocol(String),
    InvalidHistory(String),
    UnsupportedHistory { prefix: Prefix },
    InvalidReport(String),
    ArithmeticOverflow,
    Physical(crate::shared_surface::Error),
}
impl From<crate::shared_surface::Error> for Error {
    fn from(e: crate::shared_surface::Error) -> Self {
        match e {
            crate::shared_surface::Error::ArithmeticOverflow => Self::ArithmeticOverflow,
            other => Self::Physical(other),
        }
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidProtocol(s) => write!(f, "invalid active-surface protocol: {s}"),
            Self::InvalidHistory(s) => write!(f, "invalid active-surface history: {s}"),
            Self::InvalidReport(s) => write!(f, "invalid active-surface report: {s}"),
            Self::UnsupportedHistory { prefix } => write!(
                f,
                "unsupported active-surface history for Agent {:?} at {:?}",
                prefix.role, prefix.checkpoint
            ),
            Self::ArithmeticOverflow => f.write_str("active-surface exact arithmetic overflow"),
            Self::Physical(e) => e.fmt(f),
        }
    }
}
impl std::error::Error for Error {}
fn invalid<T>(message: &str) -> Result<T, Error> {
    Err(Error::InvalidHistory(message.into()))
}
