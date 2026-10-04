//! Source treatments and explicitly attributed later-port reference bundle.
use super::*;
use crate::{model::ModelConfig, presets::ModelPreset};
pub fn presets() -> Vec<ModelPreset> {
    vec![
 ("geosim-paper","GeoSim: The Size of Wars",GeosimConfig::default()),
 ("geosim-no-technology","No technological change",GeosimConfig{shock_shift:0.0,..Default::default()}),
 ("geosim-no-context","No contextual activation",GeosimConfig{context_activation:false,..Default::default()}),
 ("geosim-smaller-shocks","Smaller technological shocks",GeosimConfig{shock_shift:10.0,..Default::default()}),
 ("geosim-artifact-2017","GeoSim2 archived-port readings",GeosimConfig::artifact_2017()),
].into_iter().map(|(id,name,config)|ModelPreset{id,name,source:"Cederman2003 APSR97(1):135–150; explicitly labeled later GeoSim2 port",description:"Technology, resource capacity and contextual activation produce measurable conflict clusters. Resolved readings remain visible; abstract damage is not deaths, and the artifact bundle is not established as the exact publication implementation.",config:ModelConfig::Geosim(config)}).collect()
}
