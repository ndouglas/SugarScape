use super::super::*;

pub const DIAGNOSTIC_VERSION: &str = "active-surface-diagnostic-v1";
pub const PROTOCOL_VERSION: u16 = 1;

/// Frozen primary order followed by the separately declared misspecification settings.
pub fn frozen_settings() -> Vec<(Protocol, Environment)> {
    let mut settings = Vec::with_capacity(46);
    for model in Mechanism::ALL {
        for role in [Role::A, Role::B] {
            for kind in [
                PolicyKind::Adaptive,
                PolicyKind::FixedThree,
                PolicyKind::NoProbe,
                PolicyKind::InspectOnly,
                PolicyKind::Known,
            ] {
                settings.push((Protocol::new(role, kind), Environment::InFamily(model)));
            }
        }
    }
    for role in [Role::A, Role::B] {
        for kind in [
            PolicyKind::Adaptive,
            PolicyKind::NoProbe,
            PolicyKind::InspectOnly,
        ] {
            settings.push((Protocol::new(role, kind), Environment::DataFlip));
        }
    }
    settings
}

pub(in crate::active_surface) fn supplied_structure() -> Vec<String> {
    [
        "Four declared catalog mechanisms: shared persistent, shared resetting, private persistent, inert; DataFlip is outside the catalog.",
        "Seven symbols, opaque surface and Agent IDs, original chronological role ownership, generic write acceptance and physical resets.",
        "48 starting credits per Agent; read/write/wait cost 1; trusted inspection costs 4; correct prediction reward 12, otherwise 0.",
        "Four independent fair-bit pairs; each completed probe round costs each Agent 2; at most 3 rounds; stopping is a free public boundary.",
        "Experimenter expected remaining net task utility; Stop/Inspect tie priorities; private own routine selection and public phase length.",
        "Unknown experimenter Uniform prior; Known informed point controls; candidate responder point prior; actual DataFlip responder nominal SP.",
        "Own complete prefix and exact public clock only for local inference; no privileged environment, bits, peer history or read lineage in an Agent View.",
        "Nominal catalog certainty has Supplied/Observed provenance; out-of-catalog true-model mass and identification are unavailable.",
        "40 primary settings and 6 secondary settings; every setting has the 256 distinct sequence indices 0..255.",
        "Independent reference-v2 source SHA256 6eafbad345a3e23e312929e54ff5c9aac1538496bc1c1a73a26cd769f3300675; output SHA256 e39b6bb0f76a626892223343f7c41a9843a8622363bb421fa54d0aedfe2e9f83.",
    ].map(str::to_owned).to_vec()
}

pub(in crate::active_surface) fn own_prior(
    protocol: &Protocol,
    environment: Environment,
) -> Result<OwnPrior, Error> {
    if protocol.policy != PolicyKind::Known {
        return Ok(OwnPrior::Uniform);
    }
    match environment {
        Environment::InFamily(model) => Ok(OwnPrior::PointMass(model)),
        Environment::DataFlip => Err(Error::InvalidReport(
            "DataFlip is not a declared Known control".into(),
        )),
    }
}

/// Build each policy/prior and its both-role replay once; release it after its panels.
/// The caller's settings order is preserved through the supplied indices.
pub(in crate::active_surface) fn for_each_engine(
    settings: &[(Protocol, Environment)],
    mut visit: impl FnMut(usize, &CompiledPolicy, &Replay) -> Result<(), Error>,
) -> Result<(), Error> {
    let mut engines = Vec::<(Protocol, OwnPrior)>::new();
    for (protocol, environment) in settings {
        let key = (protocol.clone(), own_prior(protocol, *environment)?);
        if !engines.contains(&key) {
            engines.push(key);
        }
    }
    for (protocol, prior) in engines {
        let policy = CompiledPolicy::build(&protocol, prior)?;
        let replay = Replay::build(&protocol, &policy)?;
        for (index, (p, environment)) in settings.iter().enumerate() {
            if *p == protocol && own_prior(p, *environment)? == prior {
                visit(index, &policy, &replay)?;
            }
        }
    }
    Ok(())
}
