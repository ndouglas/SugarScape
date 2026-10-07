use super::{
    Action, Error, Phase, Position, PriorMode, Protocol, Role, Symbol, INITIAL_CREDITS, TRIALS,
};

impl Protocol {
    pub fn validate(&self) -> Result<(), Error> {
        self.ids.validate()?;
        if self.calibration_rounds > 3 {
            return Err(Error::InvalidProtocol(
                "calibration must contain 0..=3 rounds".into(),
            ));
        }
        if self.prior_mode == PriorMode::RestartUniform
            && self.pair.roles != [super::Knowledge::Unknown; 2]
        {
            return Err(Error::InvalidProtocol(
                "uniform restart requires two unknown Agents".into(),
            ));
        }
        Ok(())
    }

    /// Complete public opportunity order; it reveals no peer's realized action.
    pub fn schedule(&self) -> Result<Vec<(Role, Position)>, Error> {
        self.validate()?;
        let count = usize::from(self.calibration_rounds) * 4 + usize::from(TRIALS) * 12;
        let mut schedule = Vec::with_capacity(count);
        for round in 1..=self.calibration_rounds {
            append_round(&mut schedule, Phase::Calibration, round)?;
        }
        for trial in 0..TRIALS {
            for round in 1..=3 {
                append_round(&mut schedule, Phase::Live { trial }, round)?;
            }
        }
        Ok(schedule)
    }
}

fn append_round(
    schedule: &mut Vec<(Role, Position)>,
    phase: Phase,
    round: u8,
) -> Result<(), Error> {
    for slot in 1..=4 {
        let position = Position { phase, round, slot };
        schedule.push((role_at(position)?, position));
    }
    Ok(())
}

pub fn role_at(position: Position) -> Result<Role, Error> {
    position.validate()?;
    Ok(match position.slot {
        1 | 4 => Role::A,
        2 | 3 => Role::B,
        _ => unreachable!("validated slot"),
    })
}

pub const fn action_cost(action: &Action) -> u8 {
    match action {
        Action::InspectOwnTarget => 4,
        Action::Read | Action::Write(_) | Action::Wait => 1,
    }
}

/// Mechanics-independent legality; the supplied controller imposes its own policy.
pub fn validate_action(role: Role, position: Position, action: &Action) -> Result<(), Error> {
    if role_at(position)? != role {
        return Err(Error::InvalidAction(format!(
            "Agent {role:?} does not own round {} slot {}",
            position.round, position.slot
        )));
    }
    if *action == Action::Write(Symbol::Blank) {
        return Err(Error::InvalidAction(
            "blank is a reset value and cannot be written".into(),
        ));
    }
    if *action == Action::InspectOwnTarget && position.phase == Phase::Calibration {
        return Err(Error::InvalidAction(
            "own-target inspection requires a live trial".into(),
        ));
    }
    Ok(())
}

pub(super) fn remaining_credits(credits: u8, action: &Action) -> Result<u8, Error> {
    if credits > INITIAL_CREDITS {
        return Err(Error::InvalidAction(
            "credits exceed the episode's initial budget".into(),
        ));
    }
    credits
        .checked_sub(action_cost(action))
        .ok_or_else(|| Error::InvalidAction("insufficient credits for action".into()))
}
