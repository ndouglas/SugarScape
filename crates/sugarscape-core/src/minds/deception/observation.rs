//! Pure sensor adapter and bounded receiver memory, with no private policy input.
use super::state::View;
use crate::minds::caching::watching::SeenCache;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Signal {
    Cue { nominal_amount: f64 },
    VisibleTransfer { amount: f64 },
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Observation {
    pub actor: u64,
    pub site: u32,
    pub tick: u64,
    pub signal: Signal,
}

/// Called by the physical sensor; the receiver receives only its result.
pub fn perceive(
    actor: u64,
    site: u32,
    tick: u64,
    actual_transfer: f64,
    view: View,
    nominal: f64,
) -> Result<Observation, String> {
    if !actual_transfer.is_finite() || actual_transfer < 0.0 {
        return Err("observation transfer must be finite and nonnegative".into());
    }
    if !nominal.is_finite() || nominal <= 0.0 {
        return Err("observation nominal amount must be finite and positive".into());
    }
    let signal = match view {
        View::Ambiguous => Signal::Cue {
            nominal_amount: nominal,
        },
        View::Clear => Signal::VisibleTransfer {
            amount: actual_transfer,
        },
    };
    Ok(Observation {
        actor,
        site,
        tick,
        signal,
    })
}

/// Apply public evidence only. A clear zero leaves earlier evidence untouched.
pub fn update_seen(
    seen: &mut BTreeMap<(u32, u64), SeenCache>,
    obs: &Observation,
    cap: usize,
) -> Result<(), String> {
    let amount = match obs.signal {
        Signal::Cue { nominal_amount } if nominal_amount.is_finite() && nominal_amount > 0.0 => {
            nominal_amount
        }
        Signal::VisibleTransfer { amount } if amount.is_finite() && amount >= 0.0 => amount,
        _ => return Err("invalid public observation amount".into()),
    };
    if amount == 0.0 {
        return Ok(());
    }
    let key = (obs.site, obs.actor);
    let previous = seen.get(&key).map_or(0.0, |entry| entry.amount);
    let total = previous + amount;
    if !previous.is_finite() || previous < 0.0 || !total.is_finite() {
        return Err(format!(
            "seen amount overflow or invalid prior at site {} actor {}",
            obs.site, obs.actor
        ));
    }
    seen.insert(
        key,
        SeenCache {
            amount: total,
            tick: obs.tick,
        },
    );
    while seen.len() > cap {
        let oldest = *seen
            .iter()
            .min_by_key(|(&(site, actor), entry)| (entry.tick, site, actor))
            .expect("over-cap memory is nonempty")
            .0;
        seen.remove(&oldest);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::minds::{caching::watching::SeenCache, deception::state::View};
    use std::collections::BTreeMap;

    #[test]
    fn ambiguous_real_and_sham_have_identical_public_evidence() {
        let fake = perceive(1, 48, 13, 0.0, View::Ambiguous, 12.0).unwrap();
        let real = perceive(1, 48, 13, 12.0, View::Ambiguous, 12.0).unwrap();
        assert_eq!(fake, real);
    }
    #[test]
    fn clear_no_transfer_keeps_an_earlier_real_cache() {
        let mut seen = BTreeMap::from([(
            (48, 1),
            SeenCache {
                amount: 4.0,
                tick: 3,
            },
        )]);
        let obs = perceive(1, 48, 13, 0.0, View::Clear, 12.0).unwrap();
        update_seen(&mut seen, &obs, 8).unwrap();
        assert_eq!(
            seen[&(48, 1)],
            SeenCache {
                amount: 4.0,
                tick: 3
            }
        );
    }
    #[test]
    fn invalid_transfer_is_rejected_in_both_views() {
        for amount in [f64::NAN, f64::INFINITY, -1.0] {
            for view in [View::Ambiguous, View::Clear] {
                assert!(perceive(1, 48, 13, amount, view, 12.0).is_err());
            }
        }
    }
    #[test]
    fn invalid_nominal_is_rejected_in_both_views() {
        for nominal in [f64::NAN, f64::INFINITY, -1.0, 0.0] {
            for view in [View::Ambiguous, View::Clear] {
                assert!(perceive(1, 48, 13, 0.0, view, nominal).is_err());
            }
        }
    }
    #[test]
    fn positive_evidence_adds_amount_and_refreshes_tick() {
        let mut seen = BTreeMap::from([(
            (48, 1),
            SeenCache {
                amount: 4.0,
                tick: 3,
            },
        )]);
        let obs = perceive(1, 48, 13, 2.0, View::Clear, 12.0).unwrap();
        update_seen(&mut seen, &obs, 8).unwrap();
        assert_eq!(
            seen[&(48, 1)],
            SeenCache {
                amount: 6.0,
                tick: 13
            }
        );
    }
    #[test]
    fn ambiguous_evidence_uses_nominal_amount_only() {
        let mut seen = BTreeMap::new();
        let obs = perceive(1, 48, 13, 0.0, View::Ambiguous, 12.0).unwrap();
        update_seen(&mut seen, &obs, 8).unwrap();
        assert_eq!(seen[&(48, 1)].amount, 12.0);
    }
    #[test]
    fn memory_evicts_by_tick_then_site_then_actor() {
        for (cap, expected) in [
            (4, vec![(30, 1), (40, 1), (40, 2), (48, 1)]),
            (3, vec![(40, 1), (40, 2), (48, 1)]),
            (2, vec![(40, 2), (48, 1)]),
            (1, vec![(48, 1)]),
        ] {
            let mut seen = BTreeMap::from([
                (
                    (50, 1),
                    SeenCache {
                        amount: 4.0,
                        tick: 2,
                    },
                ),
                (
                    (30, 1),
                    SeenCache {
                        amount: 4.0,
                        tick: 3,
                    },
                ),
                (
                    (40, 2),
                    SeenCache {
                        amount: 4.0,
                        tick: 3,
                    },
                ),
                (
                    (40, 1),
                    SeenCache {
                        amount: 4.0,
                        tick: 3,
                    },
                ),
            ]);
            let obs = perceive(1, 48, 13, 2.0, View::Clear, 12.0).unwrap();
            update_seen(&mut seen, &obs, cap).unwrap();
            assert_eq!(seen.keys().copied().collect::<Vec<_>>(), expected);
        }
    }
    #[test]
    fn zero_capacity_keeps_no_positive_evidence() {
        let mut seen = BTreeMap::new();
        let obs = perceive(1, 48, 13, 2.0, View::Clear, 12.0).unwrap();
        update_seen(&mut seen, &obs, 0).unwrap();
        assert!(seen.is_empty());
    }
    #[test]
    fn overflow_fails_without_mutating_memory() {
        let mut seen = BTreeMap::from([(
            (48, 1),
            SeenCache {
                amount: f64::MAX,
                tick: 3,
            },
        )]);
        let before = seen.clone();
        let obs = perceive(1, 48, 13, f64::MAX, View::Clear, 12.0).unwrap();
        assert!(update_seen(&mut seen, &obs, 8).is_err());
        assert_eq!(seen, before);
    }
    #[test]
    fn malformed_public_signal_fails_without_mutating_memory() {
        for signal in [
            Signal::Cue {
                nominal_amount: 0.0,
            },
            Signal::VisibleTransfer { amount: -1.0 },
            Signal::VisibleTransfer { amount: f64::NAN },
        ] {
            let mut seen = BTreeMap::new();
            let obs = Observation {
                actor: 1,
                site: 48,
                tick: 13,
                signal,
            };
            assert!(update_seen(&mut seen, &obs, 8).is_err());
            assert!(seen.is_empty());
        }
    }
    #[test]
    fn cue_serialization_has_only_public_evidence() {
        let obs = perceive(1, 48, 13, 0.0, View::Ambiguous, 12.0).unwrap();
        assert_eq!(
            serde_json::to_value(&obs).unwrap(),
            serde_json::json!({
                "actor": 1, "site": 48, "tick": 13, "signal": { "cue": { "nominal_amount": 12.0 } }
            })
        );
        assert_eq!(
            serde_json::from_str::<Observation>(&serde_json::to_string(&obs).unwrap()).unwrap(),
            obs
        );
    }
}
