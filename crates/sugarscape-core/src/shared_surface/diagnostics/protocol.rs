//! Frozen setting order and disclosure strings, independent of measurement output.
use super::*;
pub(in crate::shared_surface) type Setting = (PanelKind, Protocol, Environment, Option<Mechanism>);
pub(in crate::shared_surface) const VERSION: &str = "shared-surface-diagnostic-v1";
pub(in crate::shared_surface) const PRIMARY_COUNT: usize = 48;
pub(in crate::shared_surface) const PANEL_COUNT: usize = 220;
pub(in crate::shared_surface) const EPISODES: usize = 256;
pub(in crate::shared_surface) fn ids() -> Ids {
    Ids {
        surface: "status-field".into(),
        agents: ["Agent-A".into(), "Agent-B".into()],
    }
}
pub(in crate::shared_surface) fn supplied_structure() -> Vec<String> {
    ["Supplied: two roles, public schedule, calibration probes, data codec, four-model catalog, priors, deterministic controller, and strict participation threshold P(SP)>1/2.",
 "Learned: catalog mechanics from complete role-local histories; no privileged actual world, other Agent history, task truth, or lineage enters LocalView.",
 "All 256 independent four-trial bit sequences have equal mass; bit 2t=X_t and bit 2t+1=Y_t; ascending sequence order.",
 "Privileged report-only fields: environment, sequence/actual task bits, read lineage, true-model scores, correctness and utility. These are evaluator outputs, never controller feedback.",
 "48 credits per Agent; read/write/wait cost 1, own-target inspection costs 4, prediction costs 0, correct prediction earns 12.",
 "The matched no-communication baseline fully inspects with perfect accuracy. Net benefit can arise only from inspection cost savings; gross correctness cannot improve.",
 "NoCommunication model scores are ungraded, with uniform catalog beliefs retained. DataFlip is outside the catalog and can be supported, confident and wrong.",
 "Any failed mass makes unconditional terminal means unavailable. Spending includes all rows; no conditional-success replacement is reported.",
 "Transfer origins are separate old Unknown/Unknown three-round calibration, without task bits, costing six credits per Agent. Restart and stale new episodes each start with fresh fields and 48 credits.",
 "Renaming is an invariance check, not a utility treatment or semantic discovery; paired sensitivity measures dependence, not reward effects."] .map(str::to_owned).to_vec()
}
pub(in crate::shared_surface) fn settings() -> Vec<Setting> {
    let mut rows = Vec::with_capacity(PANEL_COUNT);
    let primary = [
        [Knowledge::Unknown; 2],
        [Knowledge::Known; 2],
        [Knowledge::NoCommunication; 2],
    ];
    let asymmetric = [
        [Knowledge::Known, Knowledge::Unknown],
        [Knowledge::Unknown, Knowledge::Known],
    ];
    let make = |c, roles, prior_mode| Protocol {
        calibration_rounds: c,
        pair: Pair { roles },
        prior_mode,
        ids: ids(),
    };
    for (kind, pairs) in [
        (PanelKind::Primary, primary.as_slice()),
        (PanelKind::Asymmetric, asymmetric.as_slice()),
    ] {
        for m in Mechanism::ALL {
            for c in 0..4 {
                for &roles in pairs {
                    rows.push((
                        kind.clone(),
                        make(c, roles, PriorMode::Treatment),
                        Environment::InFamily(m),
                        None,
                    ));
                }
            }
        }
    }
    for kind in [PanelKind::Restart, PanelKind::Stale] {
        for old in Mechanism::ALL {
            for new in Mechanism::ALL {
                for c in 0..4 {
                    let mode = if kind == PanelKind::Restart {
                        PriorMode::RestartUniform
                    } else {
                        PriorMode::Stale(old)
                    };
                    rows.push((
                        kind.clone(),
                        make(c, [Knowledge::Unknown; 2], mode),
                        Environment::InFamily(new),
                        Some(old),
                    ));
                }
            }
        }
    }
    for c in 0..4 {
        for roles in primary {
            rows.push((
                PanelKind::DataFlip,
                make(c, roles, PriorMode::Treatment),
                Environment::DataFlip,
                None,
            ));
        }
    }
    rows
}
