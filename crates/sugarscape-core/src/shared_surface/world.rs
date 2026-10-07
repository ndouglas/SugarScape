use super::{
    protocol::remaining_credits, validate_action, Action, Environment, Error, Event, Ids, Lineage,
    Mechanism, Outcome, Phase, Position, Role, Symbol,
};

#[derive(Clone)]
struct Field {
    symbol: Symbol,
    lineage: Option<Lineage>,
}

impl Field {
    fn blank() -> Self {
        Self {
            symbol: Symbol::Blank,
            lineage: None,
        }
    }
}

/// Privileged transition seam. Controllers operate on LocalView, never on World.
#[derive(Clone)]
pub struct World {
    environment: Environment,
    ids: Ids,
    shared: Field,
    private: [Field; 2],
}

impl World {
    /// Protocol validation precedes construction at command boundaries.
    pub fn new(environment: Environment, ids: Ids) -> Self {
        Self {
            environment,
            ids,
            shared: Field::blank(),
            private: [Field::blank(), Field::blank()],
        }
    }

    pub fn apply(
        &mut self,
        role: Role,
        position: Position,
        action: &Action,
        target: Option<bool>,
        credits: u8,
    ) -> Result<Event, Error> {
        self.ids.validate()?;
        validate_action(role, position, action)?;
        let credits_after = remaining_credits(credits, action)?;
        // Resolve every fallible check before modifying any field or write origin.
        let outcome = match action {
            Action::Read => Outcome::Read(self.visible_field(role).symbol),
            Action::Write(_) => Outcome::Accepted,
            Action::Wait => Outcome::Waited,
            Action::InspectOwnTarget => Outcome::Inspected(target.ok_or_else(|| {
                Error::InvalidAction("own-target inspection requires the current target".into())
            })?),
        };
        if let Action::Write(symbol) = action {
            let stored_symbol = match (self.environment, symbol) {
                (Environment::DataFlip, Symbol::Data0) => Symbol::Data1,
                (Environment::DataFlip, Symbol::Data1) => Symbol::Data0,
                _ => *symbol,
            };
            let task_trial = match (position.phase, symbol) {
                (Phase::Live { trial }, Symbol::Data0 | Symbol::Data1) => Some(trial),
                _ => None,
            };
            let field = Field {
                symbol: stored_symbol,
                lineage: Some(Lineage {
                    writer: role,
                    write_position: position,
                    symbol: *symbol,
                    task_trial,
                }),
            };
            match self.environment {
                Environment::InFamily(Mechanism::Inert) => {}
                Environment::InFamily(Mechanism::PrivatePersistent) => {
                    self.private[role.index()] = field
                }
                Environment::InFamily(_) | Environment::DataFlip => self.shared = field,
            }
        }
        Ok(Event {
            position,
            action: *action,
            outcome,
            credits_after,
        })
    }

    /// Public reset clears fields. Local transcripts and controller memory are external.
    pub fn reset(&mut self, _phase: Phase) {
        self.shared = Field::blank();
        self.private = [Field::blank(), Field::blank()];
    }

    pub fn finish_round(&mut self) {
        if self.environment == Environment::InFamily(Mechanism::SharedResetting) {
            self.shared = Field::blank();
        }
    }

    /// Evaluator-only snapshot of the currently readable origin; capture at a read.
    pub fn read_lineage(&self, role: Role) -> Option<Lineage> {
        self.visible_field(role).lineage.clone()
    }

    fn visible_field(&self, role: Role) -> &Field {
        match self.environment {
            Environment::InFamily(Mechanism::PrivatePersistent) => &self.private[role.index()],
            _ => &self.shared,
        }
    }
}
