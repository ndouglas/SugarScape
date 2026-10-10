//! Stable probability arithmetic and versioned stream derivation for W1.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NumericIssue {
    pub field: String,
    pub detail: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Probability {
    pub hazard: f64,
    pub dose: f64,
    pub ideal: f64,
    pub realized: f64,
    pub saturated: bool,
}

/// Compute ideal p and the realized probability of the 53-bit uniform sampler.
pub fn probability(hazard: f64, dt: f64) -> Result<Probability, NumericIssue> {
    if !hazard.is_finite() || hazard < 0.0 {
        return Err(NumericIssue {
            field: "hazard".into(),
            detail: format!("hazard must be finite and nonnegative: hazard={hazard}, dt={dt}"),
        });
    }
    if !dt.is_finite() || dt <= 0.0 {
        return Err(NumericIssue {
            field: "dt".into(),
            detail: format!("dt must be finite and strictly positive: hazard={hazard}, dt={dt}"),
        });
    }
    if hazard == 0.0 {
        return Ok(Probability {
            hazard: 0.0,
            dose: 0.0,
            ideal: 0.0,
            realized: 0.0,
            saturated: false,
        });
    }
    let dose = hazard * dt;
    if !dose.is_finite() || dose == 0.0 {
        return Err(NumericIssue { field: "dose".into(), detail: format!("hazard * dt must be finite and positive; overflow or positive-product underflow: hazard={hazard}, dt={dt}") });
    }
    let ideal = -libm::expm1(-dose);
    let resolution = 2.0_f64.powi(-53);
    if !ideal.is_finite() || !(0.0..=1.0).contains(&ideal) || ideal == 0.0 {
        return Err(NumericIssue { field: "probability".into(), detail: format!("positive dose must produce a finite probability in (0, 1]: hazard={hazard}, dt={dt}, dose={dose}") });
    }
    if ideal < resolution {
        return Err(NumericIssue {
            field: "probability".into(),
            detail: format!(
                "below 53-bit resolution: hazard={hazard}, dt={dt}, dose={dose}, ideal={ideal}"
            ),
        });
    }
    let realized = libm::ceil(ideal / resolution) * resolution;
    if !realized.is_finite() || !(0.0..=1.0).contains(&realized) {
        return Err(NumericIssue { field: "probability".into(), detail: format!("realized probability must be finite and in [0, 1]: ideal={ideal}, realized={realized}") });
    }
    Ok(Probability {
        hazard,
        dose,
        ideal,
        realized,
        saturated: ideal == 1.0,
    })
}

/// FNV-1a64 over the base seed's eight little-endian bytes followed by a tag.
pub fn derive_seed(seed: u64, tag: &[u8]) -> u64 {
    let mut hash = 14_695_981_039_346_656_037_u64;
    for byte in seed.to_le_bytes().iter().chain(tag) {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(1_099_511_628_211);
    }
    hash
}

/// A uniform in [0, 1) using exactly the high 53 bits of a supplied RNG word.
pub fn uniform53(word: u64) -> f64 {
    (word >> 11) as f64 / 9_007_199_254_740_992.0
}

/// Compare a supplied word to ideal p; callers skip RNG draws for zero hazard.
pub fn chance(word: u64, probability: &Probability) -> bool {
    uniform53(word) < probability.ideal
}
