use super::*;
use crate::shared_surface::{self as physical, Knowledge, LocalPrefix, LocalView, World};

#[derive(Clone, Copy)]
enum Clock {
    Choice(Boundary),
    ProbeStart { round: u8 },
    EnterLive { trial: u8 },
    Before(Position),
    After(Position),
    Prediction { trial: u8 },
    Finished,
}

/// Privileged candidate/host runtime. It is never passed to a controller.
#[derive(Clone)]
pub struct EpisodeState {
    protocol: Protocol,
    world: World,
    bits: EpisodeBits,
    histories: [Prefix; 2],
    credits: [u8; 2],
    private_bits: [Option<bool>; 2],
    clock: Clock,
    completed: u8,
    routine: Option<Choice>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Step {
    pub histories: [Prefix; 2],
    pub events: Vec<(Role, Event)>,
    pub next: Option<Boundary>,
    pub spent_delta: [u8; 2],
}

impl EpisodeState {
    pub fn new(
        protocol: &Protocol,
        environment: Environment,
        bits: EpisodeBits,
        priors: [OwnPrior; 2],
    ) -> Result<Self, Error> {
        protocol.validate()?;
        let nominal = match environment {
            Environment::InFamily(model) => model,
            Environment::DataFlip => Mechanism::SharedPersistent,
        };
        for role in [Role::A, Role::B] {
            let expected = if role == protocol.experimenter && protocol.policy != PolicyKind::Known
            {
                OwnPrior::Uniform
            } else {
                OwnPrior::PointMass(nominal)
            };
            if priors[role.index()] != expected {
                return Err(Error::InvalidProtocol(format!(
                    "Agent {role:?} supplied prior disagrees with the declared control"
                )));
            }
        }
        Ok(Self {
            protocol: protocol.clone(),
            world: World::new(environment, protocol.ids.clone()),
            bits,
            histories: [Role::A, Role::B].map(|role| Prefix {
                role,
                ids: protocol.ids.clone(),
                own_prior: priors[role.index()],
                checkpoint: Checkpoint::Boundary(Boundary::ProbeChoice { completed: 0 }),
                entries: Vec::new(),
            }),
            credits: [48; 2],
            private_bits: [None; 2],
            clock: Clock::Choice(Boundary::ProbeChoice { completed: 0 }),
            completed: 0,
            routine: None,
        })
    }
    pub fn prefix(&self, role: Role) -> &Prefix {
        &self.histories[role.index()]
    }
    pub fn credits(&self, role: Role) -> u8 {
        self.credits[role.index()]
    }

    fn checkpoint(&mut self, checkpoint: Checkpoint) {
        for h in &mut self.histories {
            h.checkpoint = checkpoint;
        }
        if let Checkpoint::Boundary(boundary @ Boundary::TrialChoice { .. }) = checkpoint {
            let responder = 1 - self.protocol.experimenter.index();
            // Both histories denote the same clock; only the experimenter chooses.
            self.histories[responder].checkpoint = Checkpoint::BeforeSlot(
                boundary
                    .trial_position(self.protocol.experimenter)
                    .expect("checked trial boundary"),
            );
        }
    }
    fn next(&self) -> Option<Boundary> {
        if let Clock::Choice(b) = self.clock {
            Some(b)
        } else {
            None
        }
    }
    fn step(&self, before: [u8; 2], events: Vec<(Role, Event)>) -> Step {
        Step {
            histories: self.histories.clone(),
            events,
            next: self.next(),
            spent_delta: [before[0] - self.credits[0], before[1] - self.credits[1]],
        }
    }
    fn start_phase(&mut self, phase: Phase) {
        self.world.reset(phase);
        for role in [Role::A, Role::B] {
            let index = role.index();
            self.histories[index]
                .entries
                .push(Entry::Physical(LocalEntry::Reset { phase }));
            self.private_bits[index] = match phase {
                Phase::Calibration => None,
                Phase::Live { trial } => {
                    let (x, y) = self.bits.0[usize::from(trial)];
                    let bit = if role == Role::A { x } else { y };
                    self.histories[index]
                        .entries
                        .push(Entry::Physical(LocalEntry::PrivateBit { trial, bit }));
                    Some(bit)
                }
            };
        }
    }
    fn supplied_action(
        &self,
        role: Role,
        position: Position,
        knowledge: Knowledge,
    ) -> Result<Action, Error> {
        let prefix = self.prefix(role);
        let models = match prefix.own_prior {
            OwnPrior::Uniform => [Probability::new(1, 4)?; 4],
            OwnPrior::PointMass(model) => Mechanism::ALL
                .map(|m| Probability::new(u64::from(m == model), 1).expect("bounded point mass")),
        };
        let view = LocalView {
            prefix: LocalPrefix {
                role,
                ids: prefix.ids.clone(),
                own_prior: prefix.own_prior,
                checkpoint: physical::Checkpoint::Action(position),
                entries: prefix
                    .entries
                    .iter()
                    .filter_map(|e| {
                        if let Entry::Physical(e) = e {
                            Some(e.clone())
                        } else {
                            None
                        }
                    })
                    .collect(),
            },
            credits: self.credits(role),
            private_bit: self.private_bits[role.index()],
            belief: Belief {
                models,
                target: None,
            },
        };
        Ok(physical::decide(&view, knowledge)?.action)
    }
    fn action(&self, role: Role, position: Position) -> Result<Action, Error> {
        if position.phase == Phase::Calibration || role != self.protocol.experimenter {
            return self.supplied_action(role, position, Knowledge::Known);
        }
        match self.routine {
            Some(Choice::Inspect) => {
                self.supplied_action(role, position, Knowledge::NoCommunication)
            }
            Some(Choice::AttemptCommunication) => Ok(match (role, position.round, position.slot) {
                (Role::A, 1, 1) | (Role::B, 2, 3) => {
                    Action::Write(if self.private_bits[role.index()] == Some(true) {
                        Symbol::Data1
                    } else {
                        Symbol::Data0
                    })
                }
                (Role::A, 3, 1) | (Role::B, 2, 2) => Action::Read,
                _ => Action::Wait,
            }),
            _ => Err(Error::InvalidHistory(
                "experimenter has no selected live routine".into(),
            )),
        }
    }
}

/// Queue one free intervention. No paid slot, reset, or task bit is executed here.
pub fn select_choice(state: &mut EpisodeState, choice: Choice) -> Result<(), Error> {
    let Clock::Choice(boundary) = state.clock else {
        return Err(Error::InvalidHistory(
            "choice requires an unselected boundary".into(),
        ));
    };
    boundary.validate_choice(choice)?;
    let required = match choice {
        Choice::ContinueProbe => 2,
        Choice::StopProbing => 0,
        Choice::Inspect => 9,
        Choice::AttemptCommunication => 6,
    };
    if state.credits(state.protocol.experimenter) < required
        || (choice == Choice::ContinueProbe && state.credits.iter().any(|credits| *credits < 2))
    {
        return Err(Error::InvalidHistory(
            "insufficient credits for selected routine".into(),
        ));
    }
    let next = match (boundary, choice) {
        (Boundary::ProbeChoice { completed }, Choice::ContinueProbe) => Clock::ProbeStart {
            round: completed + 1,
        },
        (Boundary::ProbeChoice { .. }, Choice::StopProbing) => Clock::EnterLive { trial: 0 },
        (b @ Boundary::TrialChoice { .. }, _) => {
            Clock::Before(b.trial_position(state.protocol.experimenter)?)
        }
        _ => unreachable!("validated boundary choice"),
    };
    state.histories[state.protocol.experimenter.index()]
        .entries
        .push(Entry::OwnChoice { boundary, choice });
    if let (Boundary::ProbeChoice { completed }, Choice::StopProbing) = (boundary, choice) {
        for h in &mut state.histories {
            h.entries.push(Entry::PublicProbeStop { completed });
        }
    }
    if matches!(boundary, Boundary::TrialChoice { .. }) {
        state.routine = Some(choice);
    }
    state.clock = next;
    Ok(())
}

/// Execute exactly one paid slot or one public chronological transition.
/// A paid event always returns at AfterSlot before any round finish or later bit.
pub fn advance_one(state: &mut EpisodeState) -> Result<Step, Error> {
    let before = state.credits;
    let mut events = Vec::new();
    match state.clock {
        Clock::Choice(_) => {
            return Err(Error::InvalidHistory(
                "boundary requires a choice before advancing".into(),
            ))
        }
        Clock::Finished => return Err(Error::InvalidHistory("episode is already finished".into())),
        Clock::ProbeStart { round } => {
            if round == 1 {
                state.start_phase(Phase::Calibration);
            }
            let p = Position {
                phase: Phase::Calibration,
                round,
                slot: 1,
            };
            state.clock = Clock::Before(p);
            state.checkpoint(Checkpoint::BeforeSlot(p));
        }
        Clock::EnterLive { trial } => {
            state.start_phase(Phase::Live { trial });
            state.routine = None;
            if state.protocol.experimenter == Role::A {
                let boundary = Boundary::TrialChoice { trial };
                state.clock = Clock::Choice(boundary);
                state.checkpoint(Checkpoint::Boundary(boundary));
            } else {
                let p = Position {
                    phase: Phase::Live { trial },
                    round: 1,
                    slot: 1,
                };
                state.clock = Clock::Before(p);
                state.checkpoint(Checkpoint::BeforeSlot(p));
            }
        }
        Clock::Before(position) => {
            let role = physical::role_at(position)?;
            let action = state.action(role, position)?;
            let target = match position.phase {
                Phase::Calibration => None,
                Phase::Live { trial } => {
                    let (x, y) = state.bits.0[usize::from(trial)];
                    Some(if role == Role::A { y } else { x })
                }
            };
            // World resolves all fallible checks before touching its physical fields.
            let event = state
                .world
                .apply(role, position, &action, target, state.credits(role))?;
            state.credits[role.index()] = event.credits_after;
            state.histories[role.index()]
                .entries
                .push(Entry::Physical(LocalEntry::Action(event.clone())));
            state.clock = Clock::After(position);
            state.checkpoint(Checkpoint::AfterSlot(position));
            events.push((role, event));
        }
        Clock::After(position) => {
            if position.slot == 4 {
                state.world.finish_round();
            }
            if position.phase == Phase::Calibration && position.slot == 4 {
                state.completed = position.round;
                let boundary = Boundary::ProbeChoice {
                    completed: state.completed,
                };
                state.clock = Clock::Choice(boundary);
                state.checkpoint(Checkpoint::Boundary(boundary));
            } else if let Phase::Live { trial } = position.phase {
                if position.round == 3 && position.slot == 4 {
                    state.clock = Clock::Prediction { trial };
                    state.checkpoint(Checkpoint::Prediction { trial });
                } else {
                    advance_position(state, position);
                }
            } else {
                advance_position(state, position);
            }
        }
        Clock::Prediction { trial: 3 } => {
            state.clock = Clock::Finished;
            state.checkpoint(Checkpoint::Finished);
        }
        Clock::Prediction { trial } => {
            // Reset and issue only when actually advancing beyond Prediction.
            state.clock = Clock::EnterLive { trial: trial + 1 };
            return advance_one(state);
        }
    }
    Ok(state.step(before, events))
}

fn advance_position(state: &mut EpisodeState, previous: Position) {
    let position = if previous.slot == 4 {
        Position {
            round: previous.round + 1,
            slot: 1,
            ..previous
        }
    } else {
        Position {
            slot: previous.slot + 1,
            ..previous
        }
    };
    if matches!(position.phase, Phase::Live { .. })
        && position.round == 1
        && position.slot == 2
        && state.protocol.experimenter == Role::B
        && state.routine.is_none()
    {
        let Phase::Live { trial } = position.phase else {
            unreachable!()
        };
        let boundary = Boundary::TrialChoice { trial };
        state.clock = Clock::Choice(boundary);
        state.checkpoint(Checkpoint::Boundary(boundary));
    } else {
        state.clock = Clock::Before(position);
        state.checkpoint(Checkpoint::BeforeSlot(position));
    }
}

/// Hypothetical/construction convenience. Actual hosts check support after each
/// advance_one paid observation before authorizing any later spending.
pub fn apply_choice(state: &mut EpisodeState, choice: Choice) -> Result<Step, Error> {
    let before = state.credits;
    select_choice(state, choice)?;
    let mut events = Vec::new();
    loop {
        let step = advance_one(state)?;
        events.extend(step.events);
        if state.next().is_some() || matches!(state.clock, Clock::Finished) {
            break;
        }
    }
    Ok(state.step(before, events))
}

#[cfg(test)]
mod slot_budget_tests {
    use super::*;

    fn state() -> EpisodeState {
        EpisodeState::new(
            &Protocol::new(Role::A, PolicyKind::Adaptive),
            Environment::InFamily(Mechanism::SharedResetting),
            EpisodeBits::from_index(0).unwrap(),
            [
                OwnPrior::Uniform,
                OwnPrior::PointMass(Mechanism::SharedResetting),
            ],
        )
        .unwrap()
    }

    // Catches queueing a paid routine that cannot fit its remaining local budget.
    #[test]
    fn insufficient_routine_budget_rejects_choice_before_history_effects() {
        let mut s = state();
        apply_choice(&mut s, Choice::StopProbing).unwrap();
        s.credits[0] = 8;
        let before = s.histories.clone();
        assert!(select_choice(&mut s, Choice::Inspect).is_err());
        assert_eq!(s.histories, before);
        assert_eq!(s.credits[0], 8);
    }

    // Characterizes the adapter's budget argument and failure-before-write seam.
    #[test]
    fn failed_paid_slot_preserves_world_history_and_clock() {
        let mut s = state();
        apply_choice(&mut s, Choice::StopProbing).unwrap();
        select_choice(&mut s, Choice::AttemptCommunication).unwrap();
        s.credits[0] = 0;
        let before = s.histories.clone();
        assert!(advance_one(&mut s).is_err());
        assert_eq!(s.histories, before);
        assert_eq!(s.credits, [0, 48]);
        assert!(s.world.read_lineage(Role::A).is_none());
    }

    // Catches performing shared-resetting physics during a paid AfterSlot return.
    #[test]
    fn last_paid_round_slot_returns_before_round_reset() {
        let mut s = state();
        apply_choice(&mut s, Choice::StopProbing).unwrap();
        select_choice(&mut s, Choice::AttemptCommunication).unwrap();
        loop {
            let step = advance_one(&mut s).unwrap();
            if step
                .events
                .first()
                .is_some_and(|(_, e)| e.position.round == 1 && e.position.slot == 4)
            {
                break;
            }
        }
        assert!(s.world.read_lineage(Role::A).is_some());
        advance_one(&mut s).unwrap();
        assert!(s.world.read_lineage(Role::A).is_none());
    }
}
