use std::cmp::Ordering;

use super::{
    protocol::remaining_credits, validate_action, Action, Checkpoint, Decision, Error, Knowledge,
    LocalEntry, LocalView, Mechanism, Outcome, Phase, Position, Probability, Symbol,
};

/// Supplied policy. The only input is an Agent's own local view.
pub fn decide(view: &LocalView, knowledge: Knowledge) -> Result<Decision, Error> {
    let Checkpoint::Action(position) = view.prefix.checkpoint else {
        return Err(Error::InvalidAction(
            "decisions require a before-slot checkpoint".into(),
        ));
    };
    position.validate()?;
    validate_view(view, position.phase)?;
    let participates = view.belief.models[Mechanism::SharedPersistent.index()]
        .checked_cmp(Probability::new(1, 2)?)?
        == Ordering::Greater;
    let action = match position.phase {
        Phase::Calibration if knowledge == Knowledge::NoCommunication => Action::Wait,
        Phase::Calibration => calibration_action(view, position),
        Phase::Live { trial } => match (position.round, position.slot) {
            (1, 1) | (2, 3) => {
                if knowledge != Knowledge::NoCommunication && participates {
                    Action::Write(if view.private_bit == Some(true) { Symbol::Data1 } else { Symbol::Data0 })
                } else {
                    Action::InspectOwnTarget
                }
            }
            (2, 2) if knowledge != Knowledge::NoCommunication && participates => Action::Read,
            (3, 1) if knowledge != Knowledge::NoCommunication && view.prefix.entries.iter().any(|entry| {
                matches!(entry, LocalEntry::Action(event) if event.position == Position { phase: Phase::Live { trial }, round: 1, slot: 1 } && matches!(event.action, Action::Write(_)))
            }) => Action::Read,
            _ => Action::Wait,
        },
    };
    validate_action(view.prefix.role, position, &action)?;
    remaining_credits(view.credits, &action)?;
    Ok(Decision { action })
}

/// Exact majority at the terminal public checkpoint; a fair tie predicts zero.
pub fn predict(view: &LocalView) -> Result<bool, Error> {
    let Checkpoint::Prediction { trial } = view.prefix.checkpoint else {
        return Err(Error::InvalidAction(
            "prediction requires the terminal live checkpoint".into(),
        ));
    };
    let phase = Phase::Live { trial };
    phase.validate()?;
    validate_view(view, phase)?;
    Ok(view
        .belief
        .target
        .unwrap_or(Probability::new(1, 2)?)
        .checked_cmp(Probability::new(1, 2)?)?
        == Ordering::Greater)
}

fn calibration_action(view: &LocalView, position: Position) -> Action {
    match (position.round, position.slot) {
        (1, 1) => Action::Write(Symbol::Probe0),
        (1, 2) | (1, 4) | (2, 1) | (2, 2) | (2, 4) | (3, 2) => Action::Read,
        (1, 3) if read_was(view, 1, 2, Symbol::Probe0) => Action::Write(Symbol::Ack0),
        (2, 3) => Action::Write(Symbol::Probe1),
        (3, 1) if read_was(view, 2, 4, Symbol::Probe1) => Action::Write(Symbol::Ack1),
        _ => Action::Wait,
    }
}

fn read_was(view: &LocalView, round: u8, slot: u8, symbol: Symbol) -> bool {
    view.prefix.entries.iter().any(|entry| {
        matches!(entry, LocalEntry::Action(event)
        if event.position == Position { phase: Phase::Calibration, round, slot }
        && event.outcome == Outcome::Read(symbol))
    })
}

// Full transcript chronology is checked by Ensemble::infer. Controller-only fixtures
// can supply a partial local transcript, but its current bit, mass and budget must agree.
fn validate_view(view: &LocalView, phase: Phase) -> Result<(), Error> {
    view.prefix.ids.validate()?;
    let total = view
        .belief
        .models
        .iter()
        .try_fold(Probability::new(0, 1)?, |sum, mass| sum.checked_add(*mass))?;
    if total != Probability::new(1, 1)? {
        return Err(Error::InvalidProtocol(
            "model probabilities must sum to one".into(),
        ));
    }
    if let Some(target) = view.belief.target {
        Probability::new(target.numerator, target.denominator)?;
    }
    let credits = view
        .prefix
        .entries
        .iter()
        .rev()
        .find_map(|entry| match entry {
            LocalEntry::Action(event) => Some(event.credits_after),
            _ => None,
        })
        .unwrap_or(super::INITIAL_CREDITS);
    if view.credits != credits || view.credits > super::INITIAL_CREDITS {
        return Err(Error::InvalidAction(
            "view credits disagree with local transcript".into(),
        ));
    }
    match phase {
        Phase::Calibration if view.private_bit.is_some() => {
            return Err(Error::InvalidAction(
                "calibration has no private task bit".into(),
            ))
        }
        Phase::Live { trial } => {
            let bit = view
                .prefix
                .entries
                .iter()
                .rev()
                .find_map(|entry| match entry {
                    LocalEntry::PrivateBit { trial: issued, bit } if *issued == trial => Some(*bit),
                    _ => None,
                });
            if bit.is_none() || bit != view.private_bit {
                return Err(Error::InvalidAction(
                    "live view requires its issued current private bit".into(),
                ));
            }
        }
        _ => {}
    }
    Ok(())
}
