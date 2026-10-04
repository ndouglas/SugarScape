//! Source-labelled profiles; the switches remain visible in exports.
use super::*;
use crate::{model::ModelConfig, presets::ModelPreset};
pub fn presets() -> Vec<ModelPreset> {
    let original = PolarityConfig::default();
    let defense = PolarityConfig {
        superiority: 3.0,
        victory: 3.0,
        ..original.clone()
    };
    let alliances = PolarityConfig {
        alliances: true,
        ..original.clone()
    };
    let pra = PolarityConfig {
        source_profile: SourceProfile::Chapter5,
        allocation: Allocation::Pra,
        ..original
    };
    [("polarity-original","Emergent polarity",PolarityConfig::default()),("polarity-defense","Defense advantage",defense),("polarity-alliances","Behavioral alliances",alliances),("polarity-pra","Proportional resource allocation",pra),("polarity-two-level","Provincial revolt",PolarityConfig{predator_share:1.0,..PolarityConfig::for_variant(Variant::TwoLevel)}),("polarity-overextension","Stochastic overextension",PolarityConfig{predator_share:1.0,..PolarityConfig::for_variant(Variant::Overextension)})].into_iter().map(|(id,name,config)|ModelPreset{id,name,source:"Cederman 1994, ISQ 38(4):501–533; Cederman 1997 ch.4–5",description:"Inspectable source reconstruction. Numerical policy, timing, ties, paths, capture/transfer and provincial defaults are explicit readings; no original executable was recovered. Predator share .2 is a playground selection.",config:ModelConfig::Polarity(config)}).collect()
}
