use super::super::*;

// A changed partner policy must leave the SP responder's unreceived target fair.
#[test]
fn inspection_partner_has_fair_responder_target_and_private_choice() {
    for actor in [Role::A, Role::B] {
        let protocol = Protocol::new(actor, PolicyKind::InspectOnly);
        let policy = CompiledPolicy::build(&protocol, OwnPrior::Uniform).unwrap();
        let replay = Replay::build(&protocol, &policy).unwrap();
        let episode = run_episode(
            &protocol,
            Environment::InFamily(Mechanism::SharedPersistent),
            0,
            &policy,
            &replay,
        )
        .unwrap();
        let responder = 1 - actor.index();
        assert!(episode.histories[responder]
            .entries
            .iter()
            .all(|e| !matches!(e, Entry::OwnChoice { .. })));
        assert!(episode
            .histories
            .iter()
            .all(|h| h.entries.contains(&Entry::PublicProbeStop { completed: 0 })));
        assert!(episode.prediction_beliefs[responder]
            .iter()
            .all(|b| b.target == Some(Probability::new(1, 2).unwrap())));
        assert_eq!(episode.metrics[responder].lineage_reads, 0);
    }
}

// Contract mismatch must fail before generating candidate worlds.
#[test]
fn replay_rejects_mismatched_compiled_protocol() {
    let protocol = Protocol::new(Role::A, PolicyKind::InspectOnly);
    let policy = CompiledPolicy::build(&protocol, OwnPrior::Uniform).unwrap();
    let other = Protocol::new(Role::B, PolicyKind::InspectOnly);
    assert!(matches!(
        Replay::build(&other, &policy),
        Err(Error::InvalidProtocol(_))
    ));
}

// Replay must preserve the original-mass actor posterior at every reached clock,
// including queued free choices, paid slots, predictions and public resets.
#[test]
fn reached_actor_checkpoints_match_intervention_inference_without_future_bits() {
    for role in [Role::A, Role::B] {
        let p = Protocol::new(role, PolicyKind::Adaptive);
        let c = CompiledPolicy::build(&p, OwnPrior::Uniform).unwrap();
        let replay = Replay::build(&p, &c).unwrap();
        let mut priors = [OwnPrior::PointMass(Mechanism::SharedPersistent); 2];
        priors[role.index()] = OwnPrior::Uniform;
        let mut state = EpisodeState::new(
            &p,
            Environment::InFamily(Mechanism::SharedPersistent),
            EpisodeBits::from_index(85).unwrap(),
            priors,
        )
        .unwrap();
        let mut next = true;
        loop {
            let prefix = state.prefix(role);
            let belief = replay.infer(prefix).unwrap();
            assert_eq!(belief, c.infer(prefix).unwrap());
            if !prefix
                .entries
                .iter()
                .any(|e| matches!(e, Entry::Physical(LocalEntry::PrivateBit { .. })))
            {
                assert_eq!(belief.target, None);
            }
            if prefix.checkpoint == Checkpoint::Finished {
                break;
            }
            if next {
                let decision = c
                    .decide(&View::from_prefix(prefix.clone(), belief).unwrap())
                    .unwrap();
                select_choice(&mut state, decision.choice).unwrap();
                for r in [Role::A, Role::B] {
                    let queued = state.prefix(r);
                    let b = replay.infer(queued).unwrap();
                    if decision.choice == Choice::StopProbing {
                        assert_eq!(b.target, None);
                        assert!(View::from_prefix(queued.clone(), b)
                            .unwrap()
                            .private_bit
                            .is_none());
                    }
                }
                assert_eq!(
                    replay.infer(state.prefix(role)).unwrap(),
                    c.infer(state.prefix(role)).unwrap()
                );
            }
            next = advance_one(&mut state).unwrap().next.is_some();
        }
    }
}

// Well-shaped impossible observations and malformed callers have different errors.
#[test]
fn mismatched_host_and_malformed_inference_are_operational_errors() {
    let p = Protocol::new(Role::A, PolicyKind::InspectOnly);
    let c = CompiledPolicy::build(&p, OwnPrior::Uniform).unwrap();
    let replay = Replay::build(&p, &c).unwrap();
    let e = run_episode(&p, Environment::InFamily(Mechanism::Inert), 0, &c, &replay).unwrap();
    let mut prefix = e.histories[0].clone();
    prefix.ids.surface = "different".into();
    assert!(matches!(
        replay.infer(&prefix),
        Err(Error::InvalidHistory(_))
    ));
    assert!(run_episode(
        &p,
        Environment::InFamily(Mechanism::Inert),
        256,
        &c,
        &replay
    )
    .is_err());
    let other = Protocol::new(Role::B, PolicyKind::InspectOnly);
    assert!(matches!(
        run_episode(
            &other,
            Environment::InFamily(Mechanism::Inert),
            0,
            &c,
            &replay
        ),
        Err(Error::InvalidProtocol(_))
    ));
}
