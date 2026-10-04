use sugarscape_core::deduction::*;
fn fixture() -> ScenarioConfig {
    ScenarioConfig {
        version: 1,
        display_name: "fixture".into(),
        agents: (0..3)
            .map(|id| AgentSpec {
                id,
                grants: if id == 0 { vec![0] } else { vec![] },
                objective: if id == 0 {
                    ObjectiveTeam::Threat
                } else {
                    ObjectiveTeam::Accuser
                },
            })
            .collect(),
        capabilities: vec![CapabilitySpec {
            id: 0,
            label: "effect".into(),
            effect: Effect::SetStatus {
                status: Status::Inactive,
            },
            delay: 1,
            target: TargetRule::OtherActive,
            visibility: Visibility::Watched,
        }],
        assignment: None,
        observation: ObservationRules {
            attention_capacity: 1,
            detection_per_mille: 1000,
        },
        memory: MemoryRules {
            capacity: 64,
            retention_rounds: 8,
        },
        accusation: AccusationRules {
            enabled: true,
            budget: 1,
            eligible: vec![0, 1, 2],
            wrong_cost: WrongCost::DeactivateAccuser,
        },
        objectives: ObjectiveRules {
            capability: 0,
            accuser_team: vec![1, 2],
            non_holder_survivor_threshold: 0,
            horizon_winner: ObjectiveTeam::Threat,
        },
        max_rounds: 4,
    }
}
fn respond(e: &mut Engine, a: Action) {
    let r = e.request().unwrap();
    e.submit(TurnResponse {
        request_id: r.request_id,
        actor: r.actor,
        action: a,
    })
    .unwrap();
}
fn phase(e: &mut Engine) {
    let r = e.request().unwrap();
    while e
        .request()
        .is_some_and(|n| n.round == r.round && n.phase == r.phase)
    {
        respond(e, Action::Pass)
    }
}
#[test]
fn deduction_invalid_responses_are_atomic() {
    let mut e = Engine::new(fixture(), 7).unwrap();
    let r = e.request().unwrap();
    for (actor, id, a) in [
        (1, r.request_id, Action::Pass),
        (r.actor, 999, Action::Pass),
        (r.actor, r.request_id, Action::Watch { agents: vec![1, 1] }),
        (r.actor, r.request_id, Action::Watch { agents: vec![99] }),
        (
            r.actor,
            r.request_id,
            Action::Use {
                capability: 0,
                target: 1,
            },
        ),
    ] {
        let before = e.archive();
        assert_eq!(
            e.submit(TurnResponse {
                actor,
                request_id: id,
                action: a
            }),
            Err(ActionError::InvalidResponse)
        );
        assert_eq!(before, e.archive());
        assert_eq!(e.request(), Some(r.clone()));
    }
}
#[test]
fn deduction_delay_and_replay() {
    let mut e = Engine::new(fixture(), 19).unwrap();
    respond(&mut e, Action::Watch { agents: vec![1] });
    let restored = replay(&e.archive()).unwrap();
    assert_eq!(restored.request(), e.request());
    assert_eq!(restored.fingerprint(), e.fingerprint());
    phase(&mut e);
    respond(
        &mut e,
        Action::Use {
            capability: 0,
            target: 1,
        },
    );
    phase(&mut e);
    assert_eq!(
        e.request().unwrap().observation.roster[1].status,
        Status::Active
    );
    phase(&mut e);
    phase(&mut e);
    phase(&mut e);
    assert_eq!(
        e.request().unwrap().observation.roster[1].status,
        Status::Inactive
    );
}
#[test]
fn deduction_config_and_protocol_reject_unknowns() {
    let mut c = fixture();
    c.observation.detection_per_mille = 1001;
    assert!(Engine::new(c, 0).is_err());
    assert!(serde_json::from_str::<TurnResponse>(
        r#"{"request_id":0,"actor":0,"action":{"kind":"pass"},"hidden":1}"#
    )
    .is_err());
}
#[test]
fn deduction_replay_rejects_corruption() {
    let e = Engine::new(fixture(), 0).unwrap();
    let mut a = e.archive();
    a.fingerprint ^= 1;
    assert!(replay(&a).is_err());
    a = e.archive();
    a.protocol_version = 99;
    assert!(replay(&a).is_err());
    a = e.archive();
    a.responses.push(TurnResponse {
        request_id: 99,
        actor: 0,
        action: Action::Pass,
    });
    assert!(replay(&a).is_err());
}
#[test]
fn deduction_zero_delay_and_simultaneous_uses() {
    let mut c = fixture();
    c.capabilities[0].delay = 0;
    c.agents[1].grants = vec![0];
    let mut e = Engine::new(c, 0).unwrap();
    phase(&mut e);
    respond(
        &mut e,
        Action::Use {
            capability: 0,
            target: 1,
        },
    );
    respond(
        &mut e,
        Action::Use {
            capability: 0,
            target: 2,
        },
    );
    respond(&mut e, Action::Pass);
    assert_eq!(
        e.request()
            .unwrap()
            .observation
            .roster
            .iter()
            .map(|a| a.status)
            .collect::<Vec<_>>(),
        vec![Status::Active, Status::Inactive, Status::Inactive]
    );
}
#[test]
fn deduction_simultaneous_correct_accusations() {
    let mut e = Engine::new(fixture(), 0).unwrap();
    phase(&mut e);
    phase(&mut e);
    respond(&mut e, Action::Pass);
    respond(&mut e, Action::Accuse { target: 0 });
    respond(&mut e, Action::Accuse { target: 0 });
    assert_eq!(e.outcome().unwrap().winner, ObjectiveTeam::Accuser);
    let restored = replay(&e.archive()).unwrap();
    assert_eq!(restored.fingerprint(), e.fingerprint());
}
#[test]
fn deduction_silenced_agents_keep_physical_actions() {
    let mut c = fixture();
    c.capabilities[0].delay = 0;
    c.capabilities[0].effect = Effect::SetStatus {
        status: Status::Silenced,
    };
    let mut e = Engine::new(c, 0).unwrap();
    phase(&mut e);
    respond(
        &mut e,
        Action::Use {
            capability: 0,
            target: 1,
        },
    );
    phase(&mut e);
    respond(&mut e, Action::Pass);
    let r = e.request().unwrap();
    assert_eq!(r.actor, 1);
    assert!(!r.legal.can_say);
    let before = e.archive();
    assert_eq!(
        e.submit(TurnResponse {
            request_id: r.request_id,
            actor: 1,
            action: Action::Accuse { target: 0 }
        }),
        Err(ActionError::InvalidResponse)
    );
    assert_eq!(before, e.archive());
    phase(&mut e);
    respond(&mut e, Action::Pass);
    assert_eq!(e.request().unwrap().actor, 1);
    assert!(e.request().unwrap().legal.can_watch);
}
#[test]
fn deduction_renumbered_grant_and_revoke() {
    let mut c = fixture();
    c.capabilities[0].id = 7;
    c.agents[0].grants = vec![7, 8, 9];
    c.objectives.capability = 7;
    c.capabilities.push(CapabilitySpec {
        id: 8,
        label: "permission".into(),
        effect: Effect::Grant { capability: 7 },
        delay: 0,
        target: TargetRule::OtherActive,
        visibility: Visibility::Public,
    });
    c.capabilities.push(CapabilitySpec {
        id: 9,
        label: "remove".into(),
        effect: Effect::Revoke { capability: 7 },
        delay: 0,
        target: TargetRule::OtherActive,
        visibility: Visibility::Public,
    });
    let mut e = Engine::new(c, 0).unwrap();
    phase(&mut e);
    respond(
        &mut e,
        Action::Use {
            capability: 8,
            target: 1,
        },
    );
    phase(&mut e);
    phase(&mut e);
    phase(&mut e);
    respond(
        &mut e,
        Action::Use {
            capability: 9,
            target: 1,
        },
    );
    assert_eq!(e.request().unwrap().legal.capabilities, vec![7]);
    phase(&mut e);
    phase(&mut e);
    phase(&mut e);
    respond(&mut e, Action::Pass);
    assert!(e.request().unwrap().legal.capabilities.is_empty());
}
#[test]
fn deduction_all_config_bounds_and_references() {
    let base = fixture();
    let mut bad = vec![];
    macro_rules! case {
        ($field:ident,$value:expr) => {
            let mut c = base.clone();
            c.$field = $value;
            bad.push(c);
        };
    }
    case!(version, 99);
    case!(agents, vec![]);
    case!(capabilities, vec![]);
    case!(max_rounds, 0);
    case!(max_rounds, 1001);
    let mut c = base.clone();
    c.agents[1].id = 0;
    bad.push(c);
    let mut c = base.clone();
    c.agents[1].grants = vec![99];
    bad.push(c);
    let mut c = base.clone();
    c.capabilities.push(c.capabilities[0].clone());
    bad.push(c);
    let mut c = base.clone();
    c.capabilities[0].delay = 1001;
    bad.push(c);
    let mut c = base.clone();
    c.capabilities[0].effect = Effect::Grant { capability: 99 };
    bad.push(c);
    let mut c = base.clone();
    c.capabilities[0].effect = Effect::SetStatus {
        status: Status::Active,
    };
    bad.push(c);
    let mut c = base.clone();
    c.observation.attention_capacity = 3;
    bad.push(c);
    let mut c = base.clone();
    c.memory.capacity = 4097;
    bad.push(c);
    let mut c = base.clone();
    c.memory.retention_rounds = 1001;
    bad.push(c);
    let mut c = base.clone();
    c.accusation.budget = 33;
    bad.push(c);
    let mut c = base.clone();
    c.accusation.eligible = vec![99];
    bad.push(c);
    let mut c = base.clone();
    c.objectives.capability = 99;
    bad.push(c);
    let mut c = base.clone();
    c.objectives.accuser_team = vec![];
    bad.push(c);
    let mut c = base.clone();
    c.assignment = Some(SeededAssignment {
        capability: 0,
        eligible: vec![],
    });
    bad.push(c);
    for c in bad {
        assert!(Engine::new(c, 0).is_err());
    }
}
#[test]
fn deduction_zero_attention_memory_and_horizon() {
    let mut c = fixture();
    c.observation.attention_capacity = 0;
    c.memory.capacity = 0;
    c.max_rounds = 1;
    c.accusation.enabled = false;
    let mut e = Engine::new(c, 0).unwrap();
    while e.request().is_some() {
        respond(&mut e, Action::Pass)
    }
    assert_eq!(e.outcome().unwrap().reason, OutcomeReason::Horizon);
}
#[test]
fn deduction_enabled_accusation_requires_eligible_accuser() {
    let mut c = fixture();
    c.accusation.eligible.clear();
    assert!(Engine::new(c, 0).is_err());
}
#[test]
fn deduction_inactive_dominates_silenced_and_noop_effects() {
    let mut c = fixture();
    c.capabilities[0].delay = 0;
    c.capabilities.push(CapabilitySpec {
        id: 7,
        label: "silence".into(),
        effect: Effect::SetStatus {
            status: Status::Silenced,
        },
        delay: 0,
        target: TargetRule::OtherActive,
        visibility: Visibility::Public,
    });
    c.agents[1].grants = vec![7];
    let mut e = Engine::new(c, 0).unwrap();
    phase(&mut e);
    respond(
        &mut e,
        Action::Use {
            capability: 0,
            target: 2,
        },
    );
    respond(
        &mut e,
        Action::Use {
            capability: 7,
            target: 2,
        },
    );
    respond(&mut e, Action::Pass);
    assert_eq!(
        e.request().unwrap().observation.roster[2].status,
        Status::Inactive
    );
}
#[test]
fn deduction_objective_precedence_after_complete_discuss() {
    let mut c = fixture();
    c.objectives.non_holder_survivor_threshold = 2;
    c.max_rounds = 1;
    let mut e = Engine::new(c, 0).unwrap();
    phase(&mut e);
    phase(&mut e);
    respond(&mut e, Action::Pass);
    respond(&mut e, Action::Accuse { target: 0 });
    assert!(e.outcome().is_none());
    respond(&mut e, Action::Pass);
    assert_eq!(
        e.outcome().unwrap().reason,
        OutcomeReason::NoActiveThreatHolders
    );
}
#[test]
fn deduction_exhausted_budget_unknown_use_and_false_claim() {
    let mut c = fixture();
    c.accusation.wrong_cost = WrongCost::SpendOnly;
    let mut e = Engine::new(c, 0).unwrap();
    phase(&mut e);
    let r = e.request().unwrap();
    for action in [
        Action::Use {
            capability: 99,
            target: 1,
        },
        Action::Use {
            capability: 0,
            target: 99,
        },
    ] {
        let before = e.archive();
        assert_eq!(
            e.submit(TurnResponse {
                request_id: r.request_id,
                actor: r.actor,
                action
            }),
            Err(ActionError::InvalidResponse)
        );
        assert_eq!(before, e.archive());
    }
    phase(&mut e);
    respond(&mut e, Action::Accuse { target: 1 });
    respond(
        &mut e,
        Action::Say {
            claim: Claim::SawUse {
                source: 2,
                target: 0,
                round: 0,
            },
        },
    );
    phase(&mut e);
    phase(&mut e);
    phase(&mut e);
    let r = e.request().unwrap();
    assert_eq!(r.actor, 0);
    assert!(!r.legal.can_accuse);
    let before = e.archive();
    assert_eq!(
        e.submit(TurnResponse {
            request_id: r.request_id,
            actor: r.actor,
            action: Action::Accuse { target: 1 }
        }),
        Err(ActionError::InvalidResponse)
    );
    assert_eq!(before, e.archive());
}
#[test]
fn deduction_request_contains_only_public_rules_and_own_state() {
    let mut c = fixture();
    c.assignment = Some(SeededAssignment {
        capability: 0,
        eligible: vec![1, 2],
    });
    let e = Engine::new(c.clone(), 0).unwrap();
    let other = Engine::new(c, 1).unwrap();
    assert_eq!(
        serde_json::to_vec(&e.request()).unwrap(),
        serde_json::to_vec(&other.request()).unwrap()
    );
    let r = e.request().unwrap();
    assert_eq!(r.observation.objectives.capability, 0);
    assert_eq!(r.observation.capabilities.len(), 1);
    let text = serde_json::to_string(&r).unwrap();
    for secret in ["seed", "assigned_holder", "fingerprint", "pending"] {
        assert!(!text.contains(secret));
    }
}

#[test]
fn deduction_pass_rejects_nested_unknown_fields() {
    let input = r#"{"request_id":0,"actor":0,"action":{"kind":"pass","hidden":1}}"#;
    assert!(serde_json::from_str::<TurnResponse>(input).is_err());
    let valid = r#"{"request_id":0,"actor":0,"action":{"kind":"pass"}}"#;
    assert_eq!(
        serde_json::from_str::<TurnResponse>(valid).unwrap().action,
        Action::Pass
    );
}

#[test]
fn deduction_nested_action_claim_effect_and_event_fields_are_strict() {
    for input in [
        r#"{"kind":"watch","agents":[1],"hidden":1}"#,
        r#"{"kind":"use","capability":0,"target":1,"hidden":1}"#,
        r#"{"kind":"accuse","target":1,"hidden":1}"#,
        r#"{"kind":"say","claim":{"kind":"suspect","agent":1},"hidden":1}"#,
        r#"{"kind":"say","claim":{"kind":"suspect","agent":1,"hidden":1}}"#,
        r#"{"kind":"say","claim":{"kind":"saw_use","source":0,"target":1,"round":0,"hidden":1}}"#,
        r#"{"kind":"say","claim":{"kind":"deny_use","round":0,"hidden":1}}"#,
    ] {
        assert!(serde_json::from_str::<Action>(input).is_err(), "{input}");
    }
    for input in [
        r#"{"kind":"set_status","status":"inactive","hidden":1}"#,
        r#"{"kind":"grant","capability":0,"hidden":1}"#,
        r#"{"kind":"revoke","capability":0,"hidden":1}"#,
    ] {
        assert!(serde_json::from_str::<Effect>(input).is_err(), "{input}");
    }
    for input in [
        r#"{"kind":"use","source":0,"target":1,"capability":0,"hidden":1}"#,
        r#"{"kind":"recipient_notice","target":1,"capability":0,"hidden":1}"#,
        r#"{"kind":"status_change","agent":1,"status":"inactive","hidden":1}"#,
        r#"{"kind":"claim","source":0,"claim":{"kind":"deny_use","round":0},"hidden":1}"#,
    ] {
        assert!(
            serde_json::from_str::<EventContent>(input).is_err(),
            "{input}"
        );
    }
}

#[test]
fn deduction_partial_act_discuss_replay_and_buffered_rejection() {
    let mut e = Engine::new(fixture(), 17).unwrap();
    phase(&mut e);
    respond(
        &mut e,
        Action::Use {
            capability: 0,
            target: 1,
        },
    );
    for expected_phase in [Phase::Act, Phase::Discuss] {
        assert_eq!(e.request().unwrap().phase, expected_phase);
        let request = e.request().unwrap();
        let before = e.archive();
        assert_eq!(
            e.submit(TurnResponse {
                request_id: request.request_id + 1,
                actor: request.actor,
                action: Action::Pass
            }),
            Err(ActionError::InvalidResponse)
        );
        assert_eq!(e.archive(), before);
        let restored = replay(&before).unwrap();
        assert_eq!(restored.request(), Some(request));
        assert_eq!(restored.fingerprint(), e.fingerprint());
        if expected_phase == Phase::Act {
            phase(&mut e);
            respond(
                &mut e,
                Action::Say {
                    claim: Claim::DenyUse { round: 0 },
                },
            );
        }
    }
}

#[test]
fn deduction_existing_due_precedes_new_grant_revoke_collision() {
    for (old, new, expected_granted) in [
        (
            Effect::Grant { capability: 7 },
            Effect::Revoke { capability: 7 },
            false,
        ),
        (
            Effect::Revoke { capability: 7 },
            Effect::Grant { capability: 7 },
            true,
        ),
    ] {
        let mut c = fixture();
        c.capabilities[0].effect = old;
        c.capabilities.push(CapabilitySpec {
            id: 7,
            label: "unused".into(),
            effect: Effect::SetStatus {
                status: Status::Silenced,
            },
            delay: 0,
            target: TargetRule::OtherActive,
            visibility: Visibility::Public,
        });
        c.capabilities.push(CapabilitySpec {
            id: 8,
            label: "new".into(),
            effect: new,
            delay: 0,
            target: TargetRule::OtherActive,
            visibility: Visibility::Public,
        });
        c.agents[0].grants.push(8);
        let mut e = Engine::new(c, 0).unwrap();
        phase(&mut e);
        respond(
            &mut e,
            Action::Use {
                capability: 0,
                target: 1,
            },
        );
        phase(&mut e);
        phase(&mut e);
        phase(&mut e);
        respond(
            &mut e,
            Action::Use {
                capability: 8,
                target: 1,
            },
        );
        phase(&mut e);
        respond(&mut e, Action::Pass);
        assert_eq!(
            e.request().unwrap().observation.grants.contains(&7),
            expected_granted
        );
    }
}

#[test]
fn deduction_remaining_invalid_configuration_matrix() {
    let base = fixture();
    let mut cases = vec![];
    macro_rules! invalid {($name:literal,$c:ident,$body:block)=>{{let mut $c=base.clone();$body cases.push(($name,$c));}}}
    invalid!("too many agents", c, {
        c.agents = (0..33)
            .map(|id| AgentSpec {
                id,
                grants: if id == 0 { vec![0] } else { vec![] },
                objective: ObjectiveTeam::Accuser,
            })
            .collect();
    });
    invalid!("too many capabilities", c, {
        c.capabilities = (0..33)
            .map(|id| {
                let mut cap = c.capabilities[0].clone();
                cap.id = id;
                cap
            })
            .collect();
    });
    invalid!("duplicate grants", c, {
        c.agents[0].grants.push(0);
    });
    invalid!("duplicate accusation IDs", c, {
        c.accusation.eligible = vec![1, 1];
    });
    invalid!("duplicate objective IDs", c, {
        c.objectives.accuser_team = vec![1, 1];
    });
    invalid!("unknown objective Agent", c, {
        c.objectives.accuser_team = vec![99];
    });
    invalid!("missing initial holder", c, {
        c.agents[0].grants.clear();
    });
    invalid!("unknown revoke", c, {
        c.capabilities[0].effect = Effect::Revoke { capability: 99 };
    });
    invalid!("excess survivor threshold", c, {
        c.objectives.non_holder_survivor_threshold = 3;
    });
    invalid!("duplicate assignment", c, {
        c.assignment = Some(SeededAssignment {
            capability: 0,
            eligible: vec![1, 1],
        });
    });
    invalid!("unknown assignment Agent", c, {
        c.assignment = Some(SeededAssignment {
            capability: 0,
            eligible: vec![99],
        });
    });
    invalid!("unknown assignment capability", c, {
        c.assignment = Some(SeededAssignment {
            capability: 99,
            eligible: vec![1],
        });
    });
    invalid!("unrelated assignment capability", c, {
        let mut cap = c.capabilities[0].clone();
        cap.id = 7;
        c.capabilities.push(cap);
        c.assignment = Some(SeededAssignment {
            capability: 7,
            eligible: vec![1],
        });
    });
    for (name, c) in cases {
        assert!(Engine::new(c, 0).is_err(), "{name}");
    }
}

fn observed_use(visibility: Visibility, detection: u16, watched: AgentId) -> Engine {
    let mut c = fixture();
    c.capabilities[0].visibility = visibility;
    c.observation.detection_per_mille = detection;
    let mut e = Engine::new(c, 17).unwrap();
    respond(&mut e, Action::Pass);
    respond(
        &mut e,
        Action::Watch {
            agents: vec![watched],
        },
    );
    respond(&mut e, Action::Pass);
    respond(
        &mut e,
        Action::Use {
            capability: 0,
            target: 2,
        },
    );
    // Accepted actions remain private until the entire Act phase ends.
    assert!(e.request().unwrap().observation.events.is_empty());
    phase(&mut e);
    e
}
fn observation_for(e: &mut Engine, actor: AgentId) -> Observation {
    while e.request().unwrap().actor != actor {
        respond(e, Action::Pass);
    }
    e.request().unwrap().observation
}
#[test]
fn deduction_watched_evidence_is_selective_and_locally_numbered() {
    for (detection, watched, count) in [(1000, 0, 1), (0, 0, 0), (1000, 2, 0)] {
        let mut e = observed_use(Visibility::Watched, detection, watched);
        assert_eq!(
            e.request().unwrap().observation.events[0],
            VisibleEvent {
                sequence: 0,
                round: 0,
                content: EventContent::Use {
                    source: 0,
                    target: 2,
                    capability: 0
                },
            }
        );
        let watcher = observation_for(&mut e, 1);
        assert_eq!(watcher.events.len(), count);
        if count == 1 {
            assert_eq!(watcher.events[0].sequence, 0);
        }
        assert!(observation_for(&mut e, 2).events.is_empty());
    }
}
#[test]
fn deduction_recipient_and_public_visibility_have_no_duplicate_watched_evidence() {
    let mut recipient = observed_use(Visibility::Recipient, 1000, 0);
    assert_eq!(recipient.request().unwrap().observation.events.len(), 1);
    assert!(observation_for(&mut recipient, 1).events.is_empty());
    assert_eq!(
        observation_for(&mut recipient, 2).events[0].content,
        EventContent::RecipientNotice {
            target: 2,
            capability: 0
        }
    );
    let mut public = observed_use(Visibility::Public, 1000, 0);
    for id in 0..3 {
        assert_eq!(observation_for(&mut public, id).events.len(), 1);
    }
}
#[test]
fn deduction_public_status_has_no_cause_and_pending_target_stays_legal() {
    let mut e = observed_use(Visibility::Watched, 0, 0);
    assert!(e.request().unwrap().legal.targets.contains(&2));
    phase(&mut e); // Discuss
    phase(&mut e); // Attention
    phase(&mut e); // Act applies delayed collapse
    let r = e.request().unwrap();
    assert_eq!(r.observation.roster[2].status, Status::Inactive);
    assert_eq!(
        r.observation.events.last().unwrap().content,
        EventContent::StatusChange {
            agent: 2,
            status: Status::Inactive
        }
    );
    let json = serde_json::to_value(r.observation.events.last().unwrap()).unwrap();
    assert!(json["content"].get("source").is_none());
}
#[test]
fn deduction_false_claim_is_attributed_and_survives_eliminated_speaker() {
    let mut e = observed_use(Visibility::Watched, 0, 0);
    respond(&mut e, Action::Pass);
    respond(&mut e, Action::Pass);
    let claim = Claim::SawUse {
        source: 1,
        target: 0,
        round: 0,
    };
    respond(
        &mut e,
        Action::Say {
            claim: claim.clone(),
        },
    );
    let expected = EventContent::Claim { source: 2, claim };
    assert_eq!(
        e.request()
            .unwrap()
            .observation
            .events
            .last()
            .unwrap()
            .content,
        expected
    );
    phase(&mut e);
    phase(&mut e);
    let events = e.request().unwrap().observation.events;
    assert!(events.iter().any(|ev| ev.content == expected));
    assert!(!events
        .iter()
        .any(|ev| matches!(ev.content, EventContent::Use { source: 1, .. })));
}
#[test]
fn deduction_memory_evicts_oldest_and_preserves_age_cutoff_equality() {
    let mut c = fixture();
    c.memory.capacity = 2;
    c.memory.retention_rounds = 1;
    let mut e = Engine::new(c, 0).unwrap();
    phase(&mut e);
    phase(&mut e);
    for id in 0..3 {
        respond(
            &mut e,
            Action::Say {
                claim: Claim::Suspect { agent: id },
            },
        );
    }
    let events = e.request().unwrap().observation.events;
    assert_eq!(
        events.iter().map(|v| v.sequence).collect::<Vec<_>>(),
        vec![1, 2]
    );
    phase(&mut e);
    phase(&mut e);
    phase(&mut e);
    assert!(e.request().unwrap().observation.events.is_empty());
    // Local sequences continue after pruning, rather than exposing a world counter.
    phase(&mut e);
    phase(&mut e);
    respond(
        &mut e,
        Action::Say {
            claim: Claim::DenyUse { round: 2 },
        },
    );
    phase(&mut e);
    assert_eq!(e.request().unwrap().observation.events[0].sequence, 3);
}
#[test]
fn deduction_zero_memory_keeps_public_roster_and_zero_attention_is_valid() {
    let mut c = fixture();
    c.memory.capacity = 0;
    c.observation.attention_capacity = 0;
    c.capabilities[0].delay = 0;
    let mut e = Engine::new(c, 0).unwrap();
    phase(&mut e);
    respond(
        &mut e,
        Action::Use {
            capability: 0,
            target: 2,
        },
    );
    phase(&mut e);
    assert!(e.request().unwrap().observation.events.is_empty());
    assert_eq!(
        e.request().unwrap().observation.roster[2].status,
        Status::Inactive
    );
}
#[test]
fn deduction_wink_defaults_and_reusable_assembly() {
    let config = wink_config(6);
    assert_eq!(config.agents.len(), 6);
    assert_eq!(
        config.observation,
        ObservationRules {
            attention_capacity: 1,
            detection_per_mille: 750
        }
    );
    assert_eq!(
        config.memory,
        MemoryRules {
            capacity: 64,
            retention_rounds: 8
        }
    );
    assert_eq!(config.max_rounds, 12);
    assert_eq!(config.capabilities[0].delay, 1);
    assert_eq!(config.accusation.wrong_cost, WrongCost::DeactivateAccuser);
    assert_eq!(config.objectives.non_holder_survivor_threshold, 1);
    let e = Engine::new(config.clone(), 31).unwrap();
    let other = Engine::new(config.clone(), 31).unwrap();
    assert_eq!(e.fingerprint(), other.fingerprint());
    assert_eq!(e.request(), other.request());
    assert_eq!(replay(&e.archive()).unwrap().request(), e.request());
    let mut changed = config;
    changed.display_name = "A different assembly".into();
    changed.accusation.enabled = false;
    changed.capabilities[0].label = "signal".into();
    changed.capabilities[0].visibility = Visibility::Public;
    changed.capabilities[0].delay = 0;
    changed.max_rounds = 2;
    let mut changed = Engine::new(changed, 31).unwrap();
    while changed.request().is_some() {
        respond(&mut changed, Action::Pass);
    }
    assert_eq!(changed.outcome().unwrap().reason, OutcomeReason::Horizon);
}

#[test]
fn deduction_hidden_pending_and_buffered_choices_preserve_public_requests() {
    let mut pending = Engine::new(fixture(), 0).unwrap();
    let mut plain = pending.clone();
    phase(&mut pending);
    phase(&mut plain);
    respond(
        &mut pending,
        Action::Use {
            capability: 0,
            target: 2,
        },
    );
    respond(&mut plain, Action::Pass);
    assert_eq!(
        serde_json::to_vec(&pending.request()).unwrap(),
        serde_json::to_vec(&plain.request()).unwrap()
    );
    phase(&mut pending);
    phase(&mut plain);
    respond(&mut pending, Action::Pass);
    respond(&mut plain, Action::Pass);
    let req = pending.request().unwrap();
    assert_eq!(
        serde_json::to_vec(&req).unwrap(),
        serde_json::to_vec(&plain.request().unwrap()).unwrap()
    );
    assert!(req.legal.targets.contains(&2));
    let invalid = TurnResponse {
        request_id: req.request_id,
        actor: req.actor,
        action: Action::Use {
            capability: 0,
            target: 2,
        },
    };
    assert_eq!(pending.submit(invalid.clone()), plain.submit(invalid));
    let pending_before = pending.archive();
    let plain_before = plain.archive();
    // A buffered lie by an earlier speaker is invisible to a later speaker.
    respond(
        &mut pending,
        Action::Say {
            claim: Claim::Suspect { agent: 2 },
        },
    );
    respond(&mut plain, Action::Pass);
    assert_eq!(
        serde_json::to_vec(&pending.request()).unwrap(),
        serde_json::to_vec(&plain.request()).unwrap()
    );
    assert_eq!(replay(&pending_before).unwrap().archive(), pending_before);
    assert_eq!(replay(&plain_before).unwrap().archive(), plain_before);
}
#[test]
fn deduction_status_changes_include_wrong_accusations_and_silencing() {
    let mut c = fixture();
    c.capabilities[0].delay = 0;
    c.capabilities[0].effect = Effect::SetStatus {
        status: Status::Silenced,
    };
    c.agents[2].grants = vec![0]; // Keep an active holder after Agent 0 is eliminated.
    let mut e = Engine::new(c, 0).unwrap();
    phase(&mut e);
    respond(
        &mut e,
        Action::Use {
            capability: 0,
            target: 2,
        },
    );
    phase(&mut e);
    assert_eq!(
        e.request()
            .unwrap()
            .observation
            .events
            .last()
            .unwrap()
            .content,
        EventContent::StatusChange {
            agent: 2,
            status: Status::Silenced
        }
    );
    respond(&mut e, Action::Accuse { target: 1 });
    phase(&mut e);
    let r = e.request().unwrap();
    assert_eq!(
        r.observation.events.last().unwrap().content,
        EventContent::StatusChange {
            agent: 0,
            status: Status::Inactive
        }
    );
}
#[test]
fn deduction_wink_supported_sizes_are_valid_and_invalid_sizes_do_not_panic() {
    for n in 4..=12 {
        assert!(Engine::new(wink_config(n), 31).is_ok());
    }
    for n in [0, 1, 2, 3, 13, 32, u16::MAX] {
        assert!(Engine::new(wink_config(n), 31).is_err());
    }
}

#[test]
fn deduction_local_sequences_skip_unseen_events_without_gaps() {
    let mut e = observed_use(Visibility::Recipient, 1000, 0);
    respond(
        &mut e,
        Action::Say {
            claim: Claim::DenyUse { round: 0 },
        },
    );
    phase(&mut e);
    let first = e.request().unwrap().observation.events;
    assert_eq!(
        first.iter().map(|v| v.sequence).collect::<Vec<_>>(),
        vec![0, 1]
    );
    let observer = observation_for(&mut e, 1);
    assert_eq!(observer.events[0].sequence, 0);
    assert!(matches!(
        observer.events[0].content,
        EventContent::Claim { source: 0, .. }
    ));
}
#[test]
fn deduction_all_accepted_uses_produce_evidence_when_effects_become_noops() {
    let mut c = fixture();
    c.capabilities[0].delay = 0;
    c.capabilities[0].visibility = Visibility::Public;
    c.agents[1].grants = vec![0];
    c.agents[2].grants = vec![0];
    let mut e = Engine::new(c, 0).unwrap();
    phase(&mut e);
    respond(
        &mut e,
        Action::Use {
            capability: 0,
            target: 2,
        },
    );
    respond(
        &mut e,
        Action::Use {
            capability: 0,
            target: 2,
        },
    );
    respond(
        &mut e,
        Action::Use {
            capability: 0,
            target: 1,
        },
    );
    let events = e.request().unwrap().observation.events;
    assert_eq!(
        events
            .iter()
            .filter(|v| matches!(v.content, EventContent::Use { .. }))
            .count(),
        3
    );
    assert_eq!(
        events
            .iter()
            .filter(|v| matches!(v.content, EventContent::StatusChange { .. }))
            .count(),
        2
    );
    assert!(events.iter().any(|v| v.content
        == EventContent::Use {
            source: 2,
            target: 1,
            capability: 0
        }));
}

#[test]
fn deduction_distinct_hidden_holders_give_byte_identical_private_requests_and_errors() {
    let mut first = fixture();
    first.agents[0].grants.clear();
    first.agents[0].objective = ObjectiveTeam::Accuser;
    first.agents[1].grants = vec![0];
    first.agents[1].objective = ObjectiveTeam::Threat;
    let mut second = first.clone();
    second.agents[1].grants.clear();
    second.agents[1].objective = ObjectiveTeam::Accuser;
    second.agents[2].grants = vec![0];
    second.agents[2].objective = ObjectiveTeam::Threat;
    let mut a = Engine::new(first, 31).unwrap();
    let mut b = Engine::new(second, 31).unwrap();
    let ra = a.request().unwrap();
    let rb = b.request().unwrap();
    assert_eq!(
        serde_json::to_vec(&ra).unwrap(),
        serde_json::to_vec(&rb).unwrap()
    );
    assert_ne!(
        a.archive().config.agents[1].grants,
        b.archive().config.agents[1].grants
    );
    let invalid = TurnResponse {
        request_id: ra.request_id,
        actor: ra.actor,
        action: Action::Use {
            capability: 0,
            target: 2,
        },
    };
    assert_eq!(a.submit(invalid.clone()), b.submit(invalid));
    assert_eq!(a.request(), Some(ra));
    assert_eq!(b.request(), Some(rb));
}

#[test]
fn deduction_exact_inference_and_decision() {
    let p = posterior(&[1.0 / 3.0; 3], &[0.75, 0.25, 0.25]).unwrap();
    for (a, b) in p.iter().zip([0.6, 0.2, 0.2]) {
        assert!((a - b).abs() < 1e-12);
    }
    assert_eq!(best_accusation(&p, 1.0, -1.0, 0.0), Some(0));
    assert_eq!(best_accusation(&[1.0 / 3.0; 3], 1.0, -1.0, 0.0), None);
    assert_eq!(best_accusation(&[0.5, 0.5], 1.0, -1.0, 0.0), None);
    assert_eq!(best_accusation(&[0.5, 0.5], 1.0, 0.0, 0.0), Some(0));
    for (a, b) in [
        (vec![1.0], vec![]),
        (vec![f64::NAN], vec![1.0]),
        (vec![1.0], vec![f64::INFINITY]),
        (vec![-1.0], vec![1.0]),
        (vec![0.0], vec![1.0]),
    ] {
        assert!(posterior(&a, &b).is_err());
    }
    for p in [vec![], vec![0.2], vec![f64::NAN], vec![-1.0, 2.0]] {
        assert_eq!(best_accusation(&p, 1.0, -1.0, 0.0), None);
    }
    assert_eq!(best_accusation(&[1.0], f64::INFINITY, 0.0, 0.0), None);
}

#[test]
fn deduction_policies_accept_complete_games() {
    for kind in [
        PolicyKind::Evidence,
        PolicyKind::Random,
        PolicyKind::Reckless,
        PolicyKind::Passive,
    ] {
        for seed in 0..8 {
            let mut e = Engine::new(wink_config(6), seed).unwrap();
            let mut c: Vec<_> = (0..6)
                .map(|id| BuiltinController::new(kind, seed + id))
                .collect();
            while let Some(r) = e.request() {
                e.submit(c[usize::from(r.actor)].respond(&r)).unwrap();
            }
            assert!(e.outcome().is_some());
        }
    }
}

#[test]
fn deduction_evidence_ignores_claims_and_recipient_notices() {
    let mut e = Engine::new(wink_config(6), 4).unwrap();
    while e.request().unwrap().phase != Phase::Discuss {
        let r = e.request().unwrap();
        e.submit(TurnResponse {
            request_id: r.request_id,
            actor: r.actor,
            action: Action::Pass,
        })
        .unwrap();
    }
    let mut r = e.request().unwrap();
    r.observation.events = vec![
        VisibleEvent {
            sequence: 0,
            round: 0,
            content: EventContent::Claim {
                source: 1,
                claim: Claim::SawUse {
                    source: 2,
                    target: 3,
                    round: 0,
                },
            },
        },
        VisibleEvent {
            sequence: 1,
            round: 0,
            content: EventContent::RecipientNotice {
                target: r.actor,
                capability: 0,
            },
        },
    ];
    let mut a = BuiltinController::new(PolicyKind::Evidence, 1);
    let mut b = a.clone();
    assert_eq!(a.respond(&r).action, Action::Pass);
    assert_eq!(b.respond(&r.clone()).action, Action::Pass);
}

#[test]
fn deduction_diagnostic_is_reproducible() {
    let a = diagnose();
    let bytes = serde_json::to_string_pretty(&a).unwrap();
    assert_eq!(bytes, serde_json::to_string_pretty(&diagnose()).unwrap());
    assert!(a.perception.iter().all(|p| p.passed));
    assert!(a.inference.passed && a.decision.passed);
    assert_eq!(a.games.len(), 384);
    assert_eq!(a.frozen_config, wink_config(6));
    assert!(a.controller_seed_derivation.contains("threat=5"));
    assert_eq!(policy_seed(0, PolicyKind::Evidence, 3), 1442695040888963410);
}

#[test]
fn deduction_policies_support_self_only_sparse_capability_and_zero_resources() {
    for kind in [
        PolicyKind::Evidence,
        PolicyKind::Random,
        PolicyKind::Reckless,
        PolicyKind::Passive,
    ] {
        let mut config = wink_config(6);
        config.assignment = None;
        config.capabilities[0].id = 7;
        config.capabilities[0].target = TargetRule::SelfOnly;
        config.objectives.capability = 7;
        config.agents[0].grants = vec![7];
        config.agents[0].objective = ObjectiveTeam::Threat;
        config.observation.attention_capacity = 0;
        config.memory.capacity = 0;
        config.accusation.budget = 0;
        let mut e = Engine::new(config, 0).unwrap();
        let mut controllers: Vec<_> = (0..6).map(|i| BuiltinController::new(kind, i)).collect();
        while let Some(r) = e.request() {
            e.submit(controllers[usize::from(r.actor)].respond(&r))
                .unwrap();
        }
    }
}

#[test]
fn deduction_policies_respect_silenced_permissions() {
    let mut config = wink_config(6);
    config.assignment = None;
    config.agents[0].grants = vec![0];
    config.agents[0].objective = ObjectiveTeam::Threat;
    let mut silence = config.capabilities[0].clone();
    silence.id = 7;
    silence.delay = 0;
    silence.effect = Effect::SetStatus {
        status: Status::Silenced,
    };
    config.capabilities.push(silence);
    config.agents[1].grants = vec![7];
    let mut e = Engine::new(config, 0).unwrap();
    while let Some(r) = e.request() {
        if r.phase == Phase::Discuss && r.actor == 2 {
            assert_eq!(r.observation.roster[2].status, Status::Silenced);
            for kind in [
                PolicyKind::Evidence,
                PolicyKind::Random,
                PolicyKind::Reckless,
                PolicyKind::Passive,
            ] {
                let response = BuiltinController::new(kind, 0).respond(&r);
                assert_eq!(response.action, Action::Pass);
                e.clone().submit(response).unwrap();
            }
            break;
        }
        let action = if r.phase == Phase::Act && r.actor == 1 {
            Action::Use {
                capability: 7,
                target: 2,
            }
        } else {
            Action::Pass
        };
        e.submit(TurnResponse {
            request_id: r.request_id,
            actor: r.actor,
            action,
        })
        .unwrap();
    }
}

#[test]
fn deduction_evidence_has_no_hidden_world_input() {
    let mut a = wink_config(6);
    a.assignment = None;
    a.agents[4].grants = vec![0];
    a.agents[4].objective = ObjectiveTeam::Threat;
    let mut b = a.clone();
    b.agents[4].grants.clear();
    b.agents[4].objective = ObjectiveTeam::Accuser;
    b.agents[5].grants = vec![0];
    b.agents[5].objective = ObjectiveTeam::Threat;
    let ra = Engine::new(a, 1).unwrap().request().unwrap();
    let rb = Engine::new(b, 99).unwrap().request().unwrap();
    assert_eq!(ra, rb);
    assert_eq!(
        BuiltinController::new(PolicyKind::Evidence, 7).respond(&ra),
        BuiltinController::new(PolicyKind::Evidence, 7).respond(&rb)
    );
}

#[test]
fn deduction_evidence_prioritizes_owned_objective_over_earlier_grant() {
    for target_rule in [TargetRule::OtherActive, TargetRule::SelfOnly] {
        let mut config = wink_config(6);
        config.assignment = None;
        let mut objective = config.capabilities[0].clone();
        objective.id = 7;
        objective.target = target_rule;
        config.capabilities.push(objective);
        config.objectives.capability = 7;
        config.agents[0].grants = vec![0, 7];
        config.agents[0].objective = ObjectiveTeam::Threat;
        let mut engine = Engine::new(config, 0).unwrap();
        while engine.request().unwrap().phase == Phase::Attention {
            let r = engine.request().unwrap();
            engine
                .submit(TurnResponse {
                    request_id: r.request_id,
                    actor: r.actor,
                    action: Action::Pass,
                })
                .unwrap();
        }
        let request = engine.request().unwrap();
        let response = BuiltinController::new(PolicyKind::Evidence, 0).respond(&request);
        let target = if target_rule == TargetRule::SelfOnly {
            request.actor
        } else {
            request.legal.targets[0]
        };
        assert_eq!(
            response.action,
            Action::Use {
                capability: 7,
                target
            }
        );
        engine.submit(response).unwrap();
    }
}

#[test]
fn deduction_evidence_passes_with_only_unrelated_usable_grants() {
    let mut config = wink_config(6);
    config.assignment = None;
    let mut objective = config.capabilities[0].clone();
    objective.id = 7;
    config.capabilities.push(objective);
    config.objectives.capability = 7;
    config.agents[0].grants = vec![0];
    config.agents[1].grants = vec![7];
    config.agents[1].objective = ObjectiveTeam::Threat;
    let mut engine = Engine::new(config, 0).unwrap();
    while engine.request().unwrap().phase == Phase::Attention {
        let r = engine.request().unwrap();
        engine
            .submit(TurnResponse {
                request_id: r.request_id,
                actor: r.actor,
                action: Action::Pass,
            })
            .unwrap();
    }
    let request = engine.request().unwrap();
    let response = BuiltinController::new(PolicyKind::Evidence, 0).respond(&request);
    assert_eq!(response.action, Action::Pass);
    engine.submit(response).unwrap();
}
