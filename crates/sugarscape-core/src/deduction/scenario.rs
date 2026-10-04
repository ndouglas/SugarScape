use super::*;

/// Assemble the experimental Wink rules. The supported game size is 4..=12;
/// the returned data can be inspected and customized before engine validation.
/// Unsupported sizes produce an invalid configuration, never a panic.
pub fn wink_config(agent_count: u16) -> ScenarioConfig {
    let ids: Vec<_> = (0..agent_count.min(32)).collect();
    ScenarioConfig {
        version: RULES_VERSION,
        display_name: "Watch for the wink".into(),
        agents: ids
            .iter()
            .map(|&id| AgentSpec {
                id,
                grants: vec![],
                objective: ObjectiveTeam::Accuser,
            })
            .collect(),
        capabilities: vec![CapabilitySpec {
            id: 0,
            label: "wink".into(),
            effect: Effect::SetStatus {
                status: Status::Inactive,
            },
            delay: 1,
            target: TargetRule::OtherActive,
            visibility: Visibility::Watched,
        }],
        assignment: Some(SeededAssignment {
            capability: 0,
            eligible: ids.clone(),
        }),
        observation: ObservationRules {
            attention_capacity: 1,
            detection_per_mille: 750,
        },
        memory: MemoryRules {
            capacity: 64,
            retention_rounds: 8,
        },
        accusation: AccusationRules {
            enabled: true,
            budget: 1,
            eligible: ids.clone(),
            wrong_cost: WrongCost::DeactivateAccuser,
        },
        objectives: ObjectiveRules {
            capability: 0,
            accuser_team: ids,
            non_holder_survivor_threshold: 1,
            horizon_winner: ObjectiveTeam::Threat,
        },
        max_rounds: if (4..=12).contains(&agent_count) {
            12
        } else {
            0
        },
    }
}
