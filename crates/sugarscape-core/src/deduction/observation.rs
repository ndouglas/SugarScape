use super::*;
use rand::Rng;

impl Engine {
    pub(crate) fn remember(&mut self, observer: AgentId, content: EventContent) {
        let state = &mut self.agents[usize::from(observer)];
        state.memory.push(VisibleEvent {
            sequence: state.next_sequence,
            round: self.round,
            content,
        });
        state.next_sequence += 1;
        prune(&mut state.memory, self.round, &self.config.memory);
    }

    pub(crate) fn publish(&mut self, content: EventContent) {
        for observer in 0..self.agents.len() as AgentId {
            self.remember(observer, content.clone());
        }
    }

    pub(crate) fn observe_use(
        &mut self,
        source: AgentId,
        target: AgentId,
        capability: CapabilityId,
        visibility: Visibility,
    ) {
        let direct = EventContent::Use {
            source,
            target,
            capability,
        };
        for observer in 0..self.agents.len() as AgentId {
            // Own use is always known, and public uses have no additional
            // watched evidence or detection draws.
            if observer == source || visibility == Visibility::Public {
                self.remember(observer, direct.clone());
            } else {
                match visibility {
                    Visibility::Recipient if observer == target => {
                        self.remember(
                            observer,
                            EventContent::RecipientNotice { target, capability },
                        );
                    }
                    Visibility::Watched
                        if self.agents[usize::from(observer)]
                            .attention
                            .contains(&source)
                            && detected(
                                &mut self.rng,
                                self.config.observation.detection_per_mille,
                            ) =>
                    {
                        self.remember(observer, direct.clone());
                    }
                    _ => {}
                }
            }
        }
    }

    pub(crate) fn prune_memories(&mut self) {
        for state in &mut self.agents {
            prune(&mut state.memory, self.round, &self.config.memory);
        }
    }
}

fn detected(rng: &mut impl Rng, per_mille: u16) -> bool {
    match per_mille {
        0 => false,
        1000 => true,
        _ => rng.gen_range(0..1000u16) < per_mille,
    }
}

fn prune(memory: &mut Vec<VisibleEvent>, round: Round, rules: &MemoryRules) {
    memory.retain(|event| round.saturating_sub(event.round) <= rules.retention_rounds);
    let excess = memory.len().saturating_sub(rules.capacity);
    memory.drain(..excess);
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngCore;

    #[test]
    fn deduction_detection_endpoints_do_not_draw_randomness() {
        for (probability, expected) in [(0, false), (1000, true)] {
            let mut rng = crate::rng::seeded(31);
            let mut untouched = rng.clone();
            assert_eq!(detected(&mut rng, probability), expected);
            assert_eq!(rng.next_u64(), untouched.next_u64());
        }
    }

    #[test]
    fn deduction_observation_projection_is_pure_and_selective() {
        let mut engine = Engine::new(wink_config(6), 31).unwrap();
        let mut untouched = engine.rng.clone();
        let request = engine.request().unwrap();
        let fingerprint = engine.fingerprint();
        for _ in 0..10 {
            assert_eq!(engine.request().unwrap(), request);
            assert_eq!(engine.fingerprint(), fingerprint);
        }
        assert_eq!(engine.rng.next_u64(), untouched.next_u64());
        // Watching the recipient never samples detection for the source.
        engine.agents[1].attention = vec![2];
        let mut untouched = engine.rng.clone();
        engine.observe_use(0, 2, 0, Visibility::Watched);
        assert!(engine.agents[1].memory.is_empty());
        assert_eq!(engine.rng.next_u64(), untouched.next_u64());
    }
}
