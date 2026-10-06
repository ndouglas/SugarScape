use super::{Error, CATALOG_VERSION};
use crate::deduction::strategic_reporting::Policy;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// A disclosed integer prior weight for one reporting-policy hypothesis.
pub struct WeightedPolicy {
    pub policy: Policy,
    pub weight: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// A validated prior over at most eight structurally distinct policies.
///
/// Entries retain zero weights and are stored with canonical encodings in
/// ascending order. Weights are supplied assumptions, not inferred rationality.
pub struct Catalog {
    entries: Vec<WeightedPolicy>,
    total_weight: u16,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CatalogWire {
    version: u16,
    entries: Vec<WeightedPolicy>,
}

impl Serialize for Catalog {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        CatalogWire {
            version: CATALOG_VERSION,
            entries: self.entries.clone(),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Catalog {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = CatalogWire::deserialize(deserializer)?;
        if wire.version != CATALOG_VERSION {
            return Err(serde::de::Error::custom(
                "unsupported strategy inference catalog version",
            ));
        }
        Self::new(wire.entries).map_err(serde::de::Error::custom)
    }
}

/// Remove live rows unreachable under the policy's own calibration report.
///
/// Encoding validation precedes bit operations because `Policy::bits` is public.
pub fn canonical_bits(policy: &Policy) -> Result<u32, Error> {
    Policy::new(policy.bits)?;
    let mut bits = policy.bits & 3;
    for row in 0..16u32 {
        let signal = row & 8 != 0;
        let report = row & 4 != 0;
        if report == policy.calibration(signal) {
            bits |= policy.bits & (1 << (2 + row));
        }
    }
    Ok(bits)
}

impl Catalog {
    pub fn new(mut entries: Vec<WeightedPolicy>) -> Result<Self, Error> {
        if entries.is_empty() || entries.len() > 8 {
            return Err(Error::InvalidCatalog("requires one through eight entries"));
        }
        for entry in &mut entries {
            if entry.weight > 32 {
                return Err(Error::InvalidCatalog("each weight must be at most 32"));
            }
            entry.policy.bits = canonical_bits(&entry.policy)?;
        }
        entries.sort_by_key(|entry| entry.policy.bits);
        if entries
            .windows(2)
            .any(|pair| pair[0].policy.bits == pair[1].policy.bits)
        {
            return Err(Error::InvalidCatalog("duplicate canonical policy behavior"));
        }
        // At most eight validated u8 weights of at most 32 sum to at most 256.
        let total_weight: u16 = entries.iter().map(|entry| u16::from(entry.weight)).sum();
        if total_weight == 0 {
            return Err(Error::InvalidCatalog("total weight must be positive"));
        }
        Ok(Self {
            entries,
            total_weight,
        })
    }
    pub fn entries(&self) -> &[WeightedPolicy] {
        &self.entries
    }
    pub fn total_weight(&self) -> u16 {
        self.total_weight
    }
    pub fn uniform() -> Self {
        Self::named_controls([1, 1, 1, 1, 1])
    }
    pub fn optimization_informed() -> Self {
        Self::named_controls([1, 1, 1, 1, 16])
    }

    fn named_controls(weights: [u8; 5]) -> Self {
        let policies = [
            Policy::copy(),
            Policy::invert(),
            Policy::positive(),
            Policy::negative(),
            Policy::calibration_copy_live_invert(),
        ];
        Self::new(
            policies
                .into_iter()
                .zip(weights)
                .map(|(policy, weight)| WeightedPolicy { policy, weight })
                .collect(),
        )
        .expect("the five named controls have distinct behavior and valid prior weights")
    }
}
