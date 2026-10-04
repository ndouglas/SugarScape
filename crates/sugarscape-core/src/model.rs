//! Model kinds (milestone 9): the sugarscape and the other artificial
//! societies of Chapter VI behind one config type and one trait, so the
//! worker host, sweeps and the CLI run any of them the same way. See
//! docs/superpowers/specs/2026-09-25-other-artificial-societies-design.md.

use serde::{Serialize, Serializer};

use crate::agreement::{AgreementConfig, AgreementWorld};
use crate::anasazi::{AnasaziConfig, AnasaziWorld};
use crate::ants::{AntsConfig, AntsWorld};
use crate::auctions::{AuctionsConfig, AuctionsWorld};
use crate::bali::{BaliConfig, BaliWorld};
use crate::civil::{CivilConfig, CivilWorld};
use crate::classes::{ClassesConfig, ClassesWorld};
use crate::collusion::{CollusionConfig, CollusionWorld};
use crate::config::{Config, FieldError};
use crate::culture::{CultureConfig, CultureWorld};
use crate::democratic_peace::{DemocraticPeaceConfig, DemocraticPeaceWorld};
use crate::dpd::{DpdConfig, DpdWorld};
use crate::ethno::{EthnoConfig, EthnoWorld};
use crate::farol::{FarolConfig, FarolWorld};
use crate::firms::{FirmsConfig, FirmsWorld};
use crate::geosim::{GeosimConfig, GeosimWorld};
use crate::hoard::{HoardConfig, HoardWorld};
use crate::image::{ImageConfig, ImageWorld};
use crate::line::{LineConfig, LineWorld};
use crate::norms::{NormsConfig, NormsWorld};
use crate::opinions::{OpinionsConfig, OpinionsWorld};
use crate::polarity::{PolarityConfig, PolarityWorld};
use crate::punishment::{PunishmentConfig, PunishmentWorld};
use crate::render::{self, ColorMode, Layer};
use crate::retirement::{RetirementConfig, RetirementWorld};
use crate::ring::{RingConfig, RingWorld};
use crate::schelling::{SchellingConfig, SchellingWorld};
use crate::schema::Param;
use crate::spatial::{SpatialConfig, SpatialWorld};
use crate::structure::{StructureConfig, StructureWorld};
use crate::tags::{TagsConfig, TagsWorld};
use crate::thresholds::{ThresholdsConfig, ThresholdsWorld};
use crate::tipping::{TippingConfig, TippingWorld};
use crate::world::World;
use crate::zi::{ZiConfig, ZiWorld};
use crate::{
    agreement, anasazi, ants, bali, civil, classes, culture, dpd, ethno, export, farol, image,
    norms, opinions, punishment, retirement, ring, schelling, spatial, stats, structure, tags,
    thresholds, zi,
};

/// Which model a config or world is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelKind {
    Sugarscape,
    Schelling,
    Ring,
    Anasazi,
    Civil,
    Spatial,
    Tags,
    Culture,
    Classes,
    Ethno,
    Opinions,
    Structure,
    Dpd,
    Norms,
    Agreement,
    Image,
    Farol,
    Ants,
    Thresholds,
    Retirement,
    Punishment,
    Zi,
    Bali,
    Line,
    Tipping,
    Hoard,
    Firms,
    Collusion,
    Auctions,
    Polarity,
    Geosim,
    DemocraticPeace,
}

impl ModelKind {
    pub const ALL: [ModelKind; 32] = [
        ModelKind::Sugarscape,
        ModelKind::Schelling,
        ModelKind::Ring,
        ModelKind::Anasazi,
        ModelKind::Civil,
        ModelKind::Spatial,
        ModelKind::Tags,
        ModelKind::Culture,
        ModelKind::Classes,
        ModelKind::Ethno,
        ModelKind::Opinions,
        ModelKind::Structure,
        ModelKind::Dpd,
        ModelKind::Norms,
        ModelKind::Agreement,
        ModelKind::Image,
        ModelKind::Farol,
        ModelKind::Ants,
        ModelKind::Thresholds,
        ModelKind::Retirement,
        ModelKind::Punishment,
        ModelKind::Zi,
        ModelKind::Bali,
        ModelKind::Line,
        ModelKind::Tipping,
        ModelKind::Hoard,
        ModelKind::Firms,
        ModelKind::Collusion,
        ModelKind::Auctions,
        ModelKind::Polarity,
        ModelKind::Geosim,
        ModelKind::DemocraticPeace,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            ModelKind::Sugarscape => "sugarscape",
            ModelKind::Schelling => "schelling",
            ModelKind::Ring => "ring",
            ModelKind::Anasazi => "anasazi",
            ModelKind::Civil => "civil",
            ModelKind::Spatial => "spatial",
            ModelKind::Tags => "tags",
            ModelKind::Culture => "culture",
            ModelKind::Classes => "classes",
            ModelKind::Ethno => "ethno",
            ModelKind::Opinions => "opinions",
            ModelKind::Structure => "structure",
            ModelKind::Dpd => "dpd",
            ModelKind::Norms => "norms",
            ModelKind::Agreement => "agreement",
            ModelKind::Image => "image",
            ModelKind::Farol => "farol",
            ModelKind::Ants => "ants",
            ModelKind::Thresholds => "thresholds",
            ModelKind::Retirement => "retirement",
            ModelKind::Punishment => "punishment",
            ModelKind::Zi => "zi",
            ModelKind::Bali => "bali",
            ModelKind::Line => "line",
            ModelKind::Tipping => "tipping",
            ModelKind::Hoard => "hoard",
            ModelKind::Firms => "firms",
            ModelKind::Collusion => "collusion",
            ModelKind::Auctions => "auctions",
            ModelKind::Polarity => "polarity",
            ModelKind::Geosim => "geosim",
            ModelKind::DemocraticPeace => "democratic_peace",
        }
    }

    /// The Rules panel's fields; empty for the sugarscape, whose panel is
    /// hand-built.
    pub fn schema(self) -> Vec<Param> {
        match self {
            ModelKind::Sugarscape => Vec::new(),
            ModelKind::Schelling => schelling::schema(),
            ModelKind::Ring => ring::schema(),
            ModelKind::Anasazi => anasazi::schema(),
            ModelKind::Civil => civil::schema(),
            ModelKind::Spatial => spatial::schema(),
            ModelKind::Tags => tags::schema(),
            ModelKind::Culture => culture::schema(),
            ModelKind::Classes => classes::schema(),
            ModelKind::Ethno => ethno::schema(),
            ModelKind::Opinions => opinions::schema(),
            ModelKind::Structure => structure::schema(),
            ModelKind::Dpd => dpd::schema(),
            ModelKind::Norms => norms::schema(),
            ModelKind::Agreement => agreement::schema(),
            ModelKind::Image => image::schema(),
            ModelKind::Farol => farol::schema(),
            ModelKind::Ants => ants::schema(),
            ModelKind::Thresholds => thresholds::schema(),
            ModelKind::Retirement => retirement::schema(),
            ModelKind::Punishment => punishment::schema(),
            ModelKind::Zi => zi::schema(),
            ModelKind::Bali => bali::schema(),
            ModelKind::Line => crate::line::schema(),
            ModelKind::Tipping => crate::tipping::schema(),
            ModelKind::Hoard => crate::hoard::schema(),
            ModelKind::Firms => crate::firms::schema(),
            ModelKind::Collusion => crate::collusion::schema(),
            ModelKind::Auctions => crate::auctions::schema(),
            ModelKind::Polarity => crate::polarity::schema(),
            ModelKind::Geosim => crate::geosim::schema(),
            ModelKind::DemocraticPeace => crate::democratic_peace::schema(),
        }
    }
}

/// A config of any model. On the wire it is the model's own config object;
/// every model but the sugarscape carries `"model": "<kind>"`, and an object
/// without a `model` key (every config, link, session and sweep written
/// before milestone 9) is a sugarscape config.
// Configs are cloned rarely (never per tick), so the sugarscape's larger
// variant is not boxed.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq)]
pub enum ModelConfig {
    Sugarscape(Config),
    Schelling(SchellingConfig),
    Ring(RingConfig),
    Anasazi(AnasaziConfig),
    Civil(CivilConfig),
    Spatial(SpatialConfig),
    Tags(TagsConfig),
    Culture(CultureConfig),
    Classes(ClassesConfig),
    Ethno(EthnoConfig),
    Opinions(OpinionsConfig),
    Structure(StructureConfig),
    Dpd(DpdConfig),
    Norms(NormsConfig),
    Agreement(AgreementConfig),
    Image(ImageConfig),
    Farol(FarolConfig),
    Ants(AntsConfig),
    Thresholds(ThresholdsConfig),
    Retirement(RetirementConfig),
    Punishment(PunishmentConfig),
    Zi(ZiConfig),
    Bali(BaliConfig),
    Line(LineConfig),
    Tipping(TippingConfig),
    Hoard(HoardConfig),
    Firms(FirmsConfig),
    Collusion(CollusionConfig),
    Auctions(AuctionsConfig),
    Polarity(PolarityConfig),
    Geosim(GeosimConfig),
    DemocraticPeace(DemocraticPeaceConfig),
}

/// Another model's config on the wire: its fields and `"model": "<kind>"`.
#[derive(Serialize)]
#[serde(tag = "model", rename_all = "snake_case")]
enum Tagged<'a> {
    Schelling(&'a SchellingConfig),
    Ring(&'a RingConfig),
    Anasazi(&'a AnasaziConfig),
    Civil(&'a CivilConfig),
    Spatial(&'a SpatialConfig),
    Tags(&'a TagsConfig),
    Culture(&'a CultureConfig),
    Classes(&'a ClassesConfig),
    Ethno(&'a EthnoConfig),
    Opinions(&'a OpinionsConfig),
    Structure(&'a StructureConfig),
    Dpd(&'a DpdConfig),
    Norms(&'a NormsConfig),
    Agreement(&'a AgreementConfig),
    Image(&'a ImageConfig),
    Farol(&'a FarolConfig),
    Ants(&'a AntsConfig),
    Thresholds(&'a ThresholdsConfig),
    Retirement(&'a RetirementConfig),
    Punishment(&'a PunishmentConfig),
    Zi(&'a ZiConfig),
    Bali(&'a BaliConfig),
    Line(&'a LineConfig),
    Tipping(&'a TippingConfig),
    Hoard(&'a HoardConfig),
    Firms(&'a FirmsConfig),
    Collusion(&'a CollusionConfig),
    Auctions(&'a AuctionsConfig),
    Polarity(&'a PolarityConfig),
    Geosim(&'a GeosimConfig),
    DemocraticPeace(&'a DemocraticPeaceConfig),
}

impl From<Config> for ModelConfig {
    fn from(c: Config) -> Self {
        ModelConfig::Sugarscape(c)
    }
}

impl Serialize for ModelConfig {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            // Untagged, so sugarscape configs serialize exactly as before.
            ModelConfig::Sugarscape(c) => c.serialize(s),
            ModelConfig::Schelling(c) => Tagged::Schelling(c).serialize(s),
            ModelConfig::Ring(c) => Tagged::Ring(c).serialize(s),
            ModelConfig::Anasazi(c) => Tagged::Anasazi(c).serialize(s),
            ModelConfig::Civil(c) => Tagged::Civil(c).serialize(s),
            ModelConfig::Spatial(c) => Tagged::Spatial(c).serialize(s),
            ModelConfig::Tags(c) => Tagged::Tags(c).serialize(s),
            ModelConfig::Culture(c) => Tagged::Culture(c).serialize(s),
            ModelConfig::Classes(c) => Tagged::Classes(c).serialize(s),
            ModelConfig::Ethno(c) => Tagged::Ethno(c).serialize(s),
            ModelConfig::Opinions(c) => Tagged::Opinions(c).serialize(s),
            ModelConfig::Structure(c) => Tagged::Structure(c).serialize(s),
            ModelConfig::Dpd(c) => Tagged::Dpd(c).serialize(s),
            ModelConfig::Norms(c) => Tagged::Norms(c).serialize(s),
            ModelConfig::Agreement(c) => Tagged::Agreement(c).serialize(s),
            ModelConfig::Image(c) => Tagged::Image(c).serialize(s),
            ModelConfig::Farol(c) => Tagged::Farol(c).serialize(s),
            ModelConfig::Ants(c) => Tagged::Ants(c).serialize(s),
            ModelConfig::Thresholds(c) => Tagged::Thresholds(c).serialize(s),
            ModelConfig::Retirement(c) => Tagged::Retirement(c).serialize(s),
            ModelConfig::Punishment(c) => Tagged::Punishment(c).serialize(s),
            ModelConfig::Zi(c) => Tagged::Zi(c).serialize(s),
            ModelConfig::Bali(c) => Tagged::Bali(c).serialize(s),
            ModelConfig::Line(c) => Tagged::Line(c).serialize(s),
            ModelConfig::Tipping(c) => Tagged::Tipping(c).serialize(s),
            ModelConfig::Hoard(c) => Tagged::Hoard(c).serialize(s),
            ModelConfig::Firms(c) => Tagged::Firms(c).serialize(s),
            ModelConfig::Collusion(c) => Tagged::Collusion(c).serialize(s),
            ModelConfig::Auctions(c) => Tagged::Auctions(c).serialize(s),
            ModelConfig::Polarity(c) => Tagged::Polarity(c).serialize(s),
            ModelConfig::Geosim(c) => Tagged::Geosim(c).serialize(s),
            ModelConfig::DemocraticPeace(c) => Tagged::DemocraticPeace(c).serialize(s),
        }
    }
}

impl ModelConfig {
    pub fn kind(&self) -> ModelKind {
        match self {
            ModelConfig::Sugarscape(_) => ModelKind::Sugarscape,
            ModelConfig::Schelling(_) => ModelKind::Schelling,
            ModelConfig::Ring(_) => ModelKind::Ring,
            ModelConfig::Anasazi(_) => ModelKind::Anasazi,
            ModelConfig::Civil(_) => ModelKind::Civil,
            ModelConfig::Spatial(_) => ModelKind::Spatial,
            ModelConfig::Tags(_) => ModelKind::Tags,
            ModelConfig::Culture(_) => ModelKind::Culture,
            ModelConfig::Classes(_) => ModelKind::Classes,
            ModelConfig::Ethno(_) => ModelKind::Ethno,
            ModelConfig::Opinions(_) => ModelKind::Opinions,
            ModelConfig::Structure(_) => ModelKind::Structure,
            ModelConfig::Dpd(_) => ModelKind::Dpd,
            ModelConfig::Norms(_) => ModelKind::Norms,
            ModelConfig::Agreement(_) => ModelKind::Agreement,
            ModelConfig::Image(_) => ModelKind::Image,
            ModelConfig::Farol(_) => ModelKind::Farol,
            ModelConfig::Ants(_) => ModelKind::Ants,
            ModelConfig::Thresholds(_) => ModelKind::Thresholds,
            ModelConfig::Retirement(_) => ModelKind::Retirement,
            ModelConfig::Punishment(_) => ModelKind::Punishment,
            ModelConfig::Zi(_) => ModelKind::Zi,
            ModelConfig::Bali(_) => ModelKind::Bali,
            ModelConfig::Line(_) => ModelKind::Line,
            ModelConfig::Tipping(_) => ModelKind::Tipping,
            ModelConfig::Hoard(_) => ModelKind::Hoard,
            ModelConfig::Firms(_) => ModelKind::Firms,
            ModelConfig::Collusion(_) => ModelKind::Collusion,
            ModelConfig::Auctions(_) => ModelKind::Auctions,
            ModelConfig::Polarity(_) => ModelKind::Polarity,
            ModelConfig::Geosim(_) => ModelKind::Geosim,
            ModelConfig::DemocraticPeace(_) => ModelKind::DemocraticPeace,
        }
    }

    /// The sugarscape config, if this is one.
    pub fn sugarscape(&self) -> Option<&Config> {
        match self {
            ModelConfig::Sugarscape(c) => Some(c),
            _ => None,
        }
    }

    /// Parses and validates a config of any model.
    pub fn from_json(json: &str) -> Result<Self, Vec<FieldError>> {
        let value: serde_json::Value = serde_json::from_str(json)
            .map_err(|e| vec![FieldError::new("config", e.to_string())])?;
        let config = Self::from_value(value).map_err(|e| vec![e])?;
        config.validate()?;
        Ok(config)
    }

    /// Reads a config of any model by its `model` key: absent or
    /// `"sugarscape"` is a sugarscape config in either shape
    /// (`Config::from_value`); another model's missing fields take its
    /// defaults, and unknown fields are errors.
    pub fn from_value(mut value: serde_json::Value) -> Result<Self, FieldError> {
        let tag = match value.as_object_mut().and_then(|o| o.remove("model")) {
            None => "sugarscape".to_string(),
            Some(serde_json::Value::String(tag)) => tag,
            Some(other) => {
                return Err(FieldError::new(
                    "model",
                    format!("must be a model name, not {other}"),
                ))
            }
        };
        match tag.as_str() {
            "sugarscape" => Config::from_value(value).map(ModelConfig::Sugarscape),
            "schelling" => serde_json::from_value(value)
                .map(ModelConfig::Schelling)
                .map_err(|e| FieldError::new("config", e.to_string())),
            "ring" => serde_json::from_value(value)
                .map(ModelConfig::Ring)
                .map_err(|e| FieldError::new("config", e.to_string())),
            "anasazi" => serde_json::from_value(value)
                .map(ModelConfig::Anasazi)
                .map_err(|e| FieldError::new("config", e.to_string())),
            "civil" => serde_json::from_value(value)
                .map(ModelConfig::Civil)
                .map_err(|e| FieldError::new("config", e.to_string())),
            "spatial" => serde_json::from_value(value)
                .map(ModelConfig::Spatial)
                .map_err(|e| FieldError::new("config", e.to_string())),
            "tags" => serde_json::from_value(value)
                .map(ModelConfig::Tags)
                .map_err(|e| FieldError::new("config", e.to_string())),
            "culture" => serde_json::from_value(value)
                .map(ModelConfig::Culture)
                .map_err(|e| FieldError::new("config", e.to_string())),
            "classes" => serde_json::from_value(value)
                .map(ModelConfig::Classes)
                .map_err(|e| FieldError::new("config", e.to_string())),
            "ethno" => serde_json::from_value(value)
                .map(ModelConfig::Ethno)
                .map_err(|e| FieldError::new("config", e.to_string())),
            "opinions" => serde_json::from_value(value)
                .map(ModelConfig::Opinions)
                .map_err(|e| FieldError::new("config", e.to_string())),
            "structure" => serde_json::from_value(value)
                .map(ModelConfig::Structure)
                .map_err(|e| FieldError::new("config", e.to_string())),
            "dpd" => serde_json::from_value(value)
                .map(ModelConfig::Dpd)
                .map_err(|e| FieldError::new("config", e.to_string())),
            "norms" => serde_json::from_value(value)
                .map(ModelConfig::Norms)
                .map_err(|e| FieldError::new("config", e.to_string())),
            "agreement" => serde_json::from_value(value)
                .map(ModelConfig::Agreement)
                .map_err(|e| FieldError::new("config", e.to_string())),
            "image" => serde_json::from_value(value)
                .map(ModelConfig::Image)
                .map_err(|e| FieldError::new("config", e.to_string())),
            "farol" => serde_json::from_value(value)
                .map(ModelConfig::Farol)
                .map_err(|e| FieldError::new("config", e.to_string())),
            "ants" => serde_json::from_value(value)
                .map(ModelConfig::Ants)
                .map_err(|e| FieldError::new("config", e.to_string())),
            "thresholds" => serde_json::from_value(value)
                .map(ModelConfig::Thresholds)
                .map_err(|e| FieldError::new("config", e.to_string())),
            "retirement" => serde_json::from_value(value)
                .map(ModelConfig::Retirement)
                .map_err(|e| FieldError::new("config", e.to_string())),
            "punishment" => serde_json::from_value(value)
                .map(ModelConfig::Punishment)
                .map_err(|e| FieldError::new("config", e.to_string())),
            "line" => serde_json::from_value(value)
                .map(ModelConfig::Line)
                .map_err(|e| FieldError::new("config", e.to_string())),
            "hoard" => serde_json::from_value(value)
                .map(ModelConfig::Hoard)
                .map_err(|e| FieldError::new("config", e.to_string())),
            "tipping" => serde_json::from_value(value)
                .map(ModelConfig::Tipping)
                .map_err(|e| FieldError::new("config", e.to_string())),
            "firms" => serde_json::from_value(value)
                .map(ModelConfig::Firms)
                .map_err(|e| FieldError::new("config", e.to_string())),
            "polarity" => serde_json::from_value(value)
                .map(ModelConfig::Polarity)
                .map_err(|e| FieldError::new("config", e.to_string())),
            "geosim" => serde_json::from_value(value).map(ModelConfig::Geosim).map_err(|e|FieldError::new("config",e.to_string())),
            "democratic_peace" => serde_json::from_value(value).map(ModelConfig::DemocraticPeace).map_err(|e|FieldError::new("config",e.to_string())),
            "auctions" => serde_json::from_value(value)
                .map(ModelConfig::Auctions)
                .map_err(|e| FieldError::new("config", e.to_string())),
            "collusion" => serde_json::from_value(value)
                .map(ModelConfig::Collusion)
                .map_err(|e| FieldError::new("config", e.to_string())),
            "zi" => serde_json::from_value(value)
                .map(ModelConfig::Zi)
                .map_err(|e| FieldError::new("config", e.to_string())),
            "bali" => serde_json::from_value(value)
                .map(ModelConfig::Bali)
                .map_err(|e| FieldError::new("config", e.to_string())),
            _ => Err(FieldError::new(
                "model",
                format!(
                    "unknown model {tag:?} (expected sugarscape, schelling, ring, anasazi, civil, spatial, tags, culture, classes, ethno, opinions, structure, dpd, norms, agreement, image, farol, ants, thresholds, retirement, punishment, zi, bali, line, tipping, hoard, firms, collusion, auctions or polarity)"
                ),
            )),
        }
    }

    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        match self {
            ModelConfig::Sugarscape(c) => c.validate(),
            ModelConfig::Schelling(c) => c.validate(),
            ModelConfig::Ring(c) => c.validate(),
            ModelConfig::Anasazi(c) => c.validate(),
            ModelConfig::Civil(c) => c.validate(),
            ModelConfig::Spatial(c) => c.validate(),
            ModelConfig::Tags(c) => c.validate(),
            ModelConfig::Culture(c) => c.validate(),
            ModelConfig::Classes(c) => c.validate(),
            ModelConfig::Ethno(c) => c.validate(),
            ModelConfig::Opinions(c) => c.validate(),
            ModelConfig::Structure(c) => c.validate(),
            ModelConfig::Dpd(c) => c.validate(),
            ModelConfig::Norms(c) => c.validate(),
            ModelConfig::Agreement(c) => c.validate(),
            ModelConfig::Image(c) => c.validate(),
            ModelConfig::Farol(c) => c.validate(),
            ModelConfig::Ants(c) => c.validate(),
            ModelConfig::Thresholds(c) => c.validate(),
            ModelConfig::Retirement(c) => c.validate(),
            ModelConfig::Punishment(c) => c.validate(),
            ModelConfig::Zi(c) => c.validate(),
            ModelConfig::Bali(c) => c.validate(),
            ModelConfig::Line(c) => c.validate(),
            ModelConfig::Tipping(c) => c.validate(),
            ModelConfig::Hoard(c) => c.validate(),
            ModelConfig::Firms(c) => c.validate(),
            ModelConfig::Collusion(c) => c.validate(),
            ModelConfig::Auctions(c) => c.validate(),
            ModelConfig::Polarity(c) => c.validate(),
            ModelConfig::Geosim(c) => c.validate(),
            ModelConfig::DemocraticPeace(c) => c.validate(),
        }
    }

    /// A copy with one dotted `path` set to `value` (the model cannot be
    /// changed this way: `model` is not a field of any model's config).
    pub fn with_path(&self, path: &str, value: &serde_json::Value) -> Result<Self, FieldError> {
        match self {
            ModelConfig::Sugarscape(c) => c.with_path(path, value).map(ModelConfig::Sugarscape),
            ModelConfig::Schelling(c) => set_path(c, path, value).map(ModelConfig::Schelling),
            ModelConfig::Ring(c) => set_path(c, path, value).map(ModelConfig::Ring),
            ModelConfig::Anasazi(c) => set_path(c, path, value).map(ModelConfig::Anasazi),
            ModelConfig::Civil(c) => set_path(c, path, value).map(ModelConfig::Civil),
            ModelConfig::Spatial(c) => set_path(c, path, value).map(ModelConfig::Spatial),
            ModelConfig::Tags(c) => set_path(c, path, value).map(ModelConfig::Tags),
            ModelConfig::Culture(c) => set_path(c, path, value).map(ModelConfig::Culture),
            ModelConfig::Classes(c) => set_path(c, path, value).map(ModelConfig::Classes),
            ModelConfig::Ethno(c) => set_path(c, path, value).map(ModelConfig::Ethno),
            ModelConfig::Opinions(c) => set_path(c, path, value).map(ModelConfig::Opinions),
            ModelConfig::Structure(c) => set_path(c, path, value).map(ModelConfig::Structure),
            ModelConfig::Dpd(c) => set_path(c, path, value).map(ModelConfig::Dpd),
            ModelConfig::Norms(c) => set_path(c, path, value).map(ModelConfig::Norms),
            ModelConfig::Agreement(c) => set_path(c, path, value).map(ModelConfig::Agreement),
            ModelConfig::Image(c) => set_path(c, path, value).map(ModelConfig::Image),
            ModelConfig::Farol(c) => set_path(c, path, value).map(ModelConfig::Farol),
            ModelConfig::Ants(c) => set_path(c, path, value).map(ModelConfig::Ants),
            ModelConfig::Thresholds(c) => set_path(c, path, value).map(ModelConfig::Thresholds),
            ModelConfig::Retirement(c) => set_path(c, path, value).map(ModelConfig::Retirement),
            ModelConfig::Punishment(c) => set_path(c, path, value).map(ModelConfig::Punishment),
            ModelConfig::Zi(c) => set_path(c, path, value).map(ModelConfig::Zi),
            ModelConfig::Bali(c) => set_path(c, path, value).map(ModelConfig::Bali),
            ModelConfig::Line(c) => set_path(c, path, value).map(ModelConfig::Line),
            ModelConfig::Tipping(c) => set_path(c, path, value).map(ModelConfig::Tipping),
            ModelConfig::Hoard(c) => set_path(c, path, value).map(ModelConfig::Hoard),
            ModelConfig::Firms(c) => set_path(c, path, value).map(ModelConfig::Firms),
            ModelConfig::Collusion(c) => set_path(c, path, value).map(ModelConfig::Collusion),
            ModelConfig::Auctions(c) => set_path(c, path, value).map(ModelConfig::Auctions),
            ModelConfig::Polarity(c) => set_path(c, path, value).map(ModelConfig::Polarity),
            ModelConfig::Geosim(c) => set_path(c, path, value).map(ModelConfig::Geosim),
            ModelConfig::DemocraticPeace(c) => {
                set_path(c, path, value).map(ModelConfig::DemocraticPeace)
            }
        }
    }

    /// The most ticks a world with this config can run, when it stops on
    /// its own (the anasazi at its end year); `None` when it runs forever.
    pub fn max_ticks(&self) -> Option<u32> {
        match self {
            ModelConfig::Polarity(c) => {
                Some(c.horizon.div_ceil(u64::from(c.periods_per_tick)) as u32)
            }
            ModelConfig::DemocraticPeace(c) => {
                Some(c.horizon().div_ceil(u64::from(c.periods_per_tick)) as u32)
            }
            ModelConfig::Geosim(c) => {
                Some(c.horizon().div_ceil(u64::from(c.periods_per_tick)) as u32)
            }
            ModelConfig::Anasazi(c) => Some(c.end_year.saturating_sub(c.start_year)),
            ModelConfig::Tags(c) => (c.end > 0).then_some(c.end),
            ModelConfig::Ethno(c) => (c.end > 0).then_some(c.end),
            ModelConfig::Dpd(c) => (c.end > 0).then_some(c.end),
            ModelConfig::Image(c) => (c.end > 0).then_some(c.end),
            ModelConfig::Sugarscape(_)
            | ModelConfig::Schelling(_)
            | ModelConfig::Ring(_)
            | ModelConfig::Civil(_)
            | ModelConfig::Spatial(_)
            | ModelConfig::Culture(_)
            | ModelConfig::Classes(_)
            | ModelConfig::Opinions(_)
            | ModelConfig::Structure(_)
            | ModelConfig::Norms(_)
            | ModelConfig::Agreement(_)
            | ModelConfig::Farol(_)
            | ModelConfig::Ants(_)
            | ModelConfig::Thresholds(_)
            | ModelConfig::Retirement(_)
            | ModelConfig::Punishment(_)
            | ModelConfig::Zi(_)
            | ModelConfig::Bali(_)
            | ModelConfig::Line(_)
            | ModelConfig::Tipping(_)
            | ModelConfig::Hoard(_)
            | ModelConfig::Firms(_)
            | ModelConfig::Collusion(_)
            | ModelConfig::Auctions(_) => None,
        }
    }

    /// The statistics series a world with this config records.
    pub fn series_names(&self) -> Vec<String> {
        match self {
            ModelConfig::Sugarscape(c) => stats::series_names(c),
            ModelConfig::Schelling(_) => schelling::SERIES.iter().map(|s| s.to_string()).collect(),
            ModelConfig::Ring(_) => ring::SERIES.iter().map(|s| s.to_string()).collect(),
            ModelConfig::Anasazi(_) => anasazi::SERIES.iter().map(|s| s.to_string()).collect(),
            ModelConfig::Civil(_) => civil::SERIES.iter().map(|s| s.to_string()).collect(),
            ModelConfig::Spatial(_) => spatial::SERIES.iter().map(|s| s.to_string()).collect(),
            ModelConfig::Tags(_) => tags::SERIES.iter().map(|s| s.to_string()).collect(),
            ModelConfig::Culture(_) => culture::SERIES.iter().map(|s| s.to_string()).collect(),
            ModelConfig::Classes(_) => classes::SERIES.iter().map(|s| s.to_string()).collect(),
            ModelConfig::Ethno(_) => ethno::SERIES.iter().map(|s| s.to_string()).collect(),
            ModelConfig::Opinions(_) => opinions::SERIES.iter().map(|s| s.to_string()).collect(),
            ModelConfig::Structure(_) => structure::SERIES.iter().map(|s| s.to_string()).collect(),
            ModelConfig::Dpd(_) => dpd::SERIES.iter().map(|s| s.to_string()).collect(),
            ModelConfig::Norms(_) => norms::SERIES.iter().map(|s| s.to_string()).collect(),
            ModelConfig::Agreement(_) => agreement::SERIES.iter().map(|s| s.to_string()).collect(),
            ModelConfig::Image(_) => image::SERIES.iter().map(|s| s.to_string()).collect(),
            ModelConfig::Farol(_) => farol::SERIES.iter().map(|s| s.to_string()).collect(),
            ModelConfig::Ants(_) => ants::SERIES.iter().map(|s| s.to_string()).collect(),
            ModelConfig::Thresholds(_) => {
                thresholds::SERIES.iter().map(|s| s.to_string()).collect()
            }
            ModelConfig::Retirement(_) => {
                retirement::SERIES.iter().map(|s| s.to_string()).collect()
            }
            ModelConfig::Punishment(_) => {
                punishment::SERIES.iter().map(|s| s.to_string()).collect()
            }
            ModelConfig::Zi(_) => zi::SERIES.iter().map(|s| s.to_string()).collect(),
            ModelConfig::Bali(_) => bali::SERIES.iter().map(|s| s.to_string()).collect(),
            ModelConfig::Line(_) => crate::line::SERIES.iter().map(|s| s.to_string()).collect(),
            ModelConfig::Hoard(_) => crate::hoard::SERIES.iter().map(|s| s.to_string()).collect(),
            ModelConfig::Tipping(_) => crate::tipping::SERIES
                .iter()
                .map(|s| s.to_string())
                .collect(),
            ModelConfig::Firms(_) => crate::firms::SERIES.iter().map(|s| s.to_string()).collect(),
            ModelConfig::Auctions(c) => crate::auctions::series_names(c.bidders),
            ModelConfig::Polarity(_) => crate::polarity::series_names(),
            ModelConfig::Geosim(_) => crate::geosim::series_names(),
            ModelConfig::DemocraticPeace(_) => crate::democratic_peace::series_names(),
            ModelConfig::Collusion(_) => crate::collusion::SERIES
                .iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }
}

/// `config` with the dotted `path` set to `value` (through its JSON, like
/// `Config::with_path`; the error field is `schedule`, as there).
fn set_path<T: Serialize + serde::de::DeserializeOwned>(
    config: &T,
    path: &str,
    value: &serde_json::Value,
) -> Result<T, FieldError> {
    let mut json = serde_json::to_value(config).expect("config serializes");
    let unknown = || FieldError::new("schedule", format!("unknown field {path}"));
    let mut slot = &mut json;
    for key in path.split('.') {
        slot = slot.get_mut(key).ok_or_else(unknown)?;
    }
    *slot = value.clone();
    serde_json::from_value(json).map_err(|e| FieldError::new("schedule", format!("{path}: {e}")))
}

/// What the host, sweeps and the CLI need from a running model. `run` is
/// the spec's `step(n)`, named after `World::run` so it cannot be mistaken
/// for `World::step` (one tick).
pub trait Model {
    /// The live config (after scheduled changes and `set_config`).
    fn config(&self) -> ModelConfig;
    /// Runs `ticks` ticks.
    fn run(&mut self, ticks: u32);
    /// Completed ticks.
    fn tick(&self) -> u64;
    fn population(&self) -> usize;
    /// A hash of the full dynamic state (the golden tests' fingerprint).
    fn fingerprint(&self) -> u64;
    /// The rendered frame's width and height in cells.
    fn size(&self) -> (u32, u32);
    /// Renders the frame as RGBA into `buf` (`size()` cells). `mode` and
    /// `layer` name the model's color mode and landscape layer; models
    /// without them ignore them.
    fn render(&self, mode: &str, layer: &str, buf: &mut Vec<u8>) -> Result<(), String>;
    /// The latest statistics snapshot as JSON.
    fn latest_json(&self) -> String;
    fn series_names(&self) -> Vec<String>;
    /// The full history of series `name` (or `"tick"`), or `None` if unknown.
    fn series(&self, name: &str) -> Option<Vec<f64>>;
    /// The latest value of series `name` (or `"tick"`), or `None` if unknown or there is no history yet.
    fn latest_value(&self, name: &str) -> Option<f64> {
        self.series(name).and_then(|v| v.last().copied())
    }
    /// The statistics history as CSV (`tick`, then `series_names`).
    fn series_csv(&self) -> String;
    /// The agents alive now as CSV.
    fn agents_csv(&self) -> String;
    /// The site (x, y) and its agent as JSON.
    fn inspect_json(&self, x: u32, y: u32) -> Result<String, String>;
    /// Where agent `id` is, in frame cells, while it lives.
    fn locate(&self, id: u64) -> Option<(u32, u32)>;
    /// Applies a changed config to the running world; fields that change
    /// only on reset are refused.
    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>>;
    /// Whether the world has run its course (the anasazi's end year):
    /// `run` then does nothing. Other models never finish.
    fn finished(&self) -> bool {
        false
    }
    /// Whether a finished world's state is permanent, so that the ticks it did
    /// not run would repeat its last values (Axelrod's stable lattice; a
    /// Sugarscape whose Axelrod cultures settled). Sweeps read such a world at
    /// its last values; any other world that stopped reads NaN past its end.
    fn holds_when_finished(&self) -> bool {
        false
    }
}

/// The error for handing a world another model's config.
pub(crate) fn wrong_model(want: ModelKind, got: &ModelConfig) -> Vec<FieldError> {
    vec![FieldError::new(
        "model",
        format!(
            "a {} world cannot take a {} config",
            want.as_str(),
            got.kind().as_str()
        ),
    )]
}

impl Model for World {
    fn config(&self) -> ModelConfig {
        ModelConfig::Sugarscape(self.config.clone())
    }

    fn run(&mut self, ticks: u32) {
        World::run(self, ticks);
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    fn population(&self) -> usize {
        World::population(self)
    }

    fn fingerprint(&self) -> u64 {
        World::fingerprint(self)
    }

    fn size(&self) -> (u32, u32) {
        (self.torus.width, self.torus.height)
    }

    fn render(&self, mode: &str, layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        let mode: ColorMode = mode.parse()?;
        let layer: Layer = layer.parse()?;
        render::render(self, mode, layer, buf)
    }

    fn latest_json(&self) -> String {
        serde_json::to_string(&self.stats.latest()).expect("snapshot serializes")
    }

    fn series_names(&self) -> Vec<String> {
        stats::series_names(&self.config)
    }

    fn series(&self, name: &str) -> Option<Vec<f64>> {
        self.stats.series(name)
    }

    fn latest_value(&self, name: &str) -> Option<f64> {
        self.stats.latest().and_then(|s| s.value(name))
    }

    fn series_csv(&self) -> String {
        export::series_csv(self)
    }

    fn agents_csv(&self) -> String {
        export::agents_csv(self)
    }

    fn inspect_json(&self, x: u32, y: u32) -> Result<String, String> {
        let inspection = self.inspect(x, y)?;
        Ok(serde_json::to_string(&inspection).expect("inspection serializes"))
    }

    fn locate(&self, id: u64) -> Option<(u32, u32)> {
        World::locate(self, id).map(|p| (p.x, p.y))
    }

    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        match next {
            ModelConfig::Sugarscape(c) => World::set_config(self, c),
            other => Err(wrong_model(ModelKind::Sugarscape, &other)),
        }
    }

    fn finished(&self) -> bool {
        self.is_finished()
    }

    fn holds_when_finished(&self) -> bool {
        true
    }
}

/// A world of any model. Sugarscape-only calls (painting, networks, the
/// credit graph …) go through `sugarscape()`/`sugarscape_mut()`.
pub enum ModelWorld {
    Sugarscape(Box<World>),
    Schelling(Box<SchellingWorld>),
    Ring(Box<RingWorld>),
    Anasazi(Box<AnasaziWorld>),
    Civil(Box<CivilWorld>),
    Spatial(Box<SpatialWorld>),
    Tags(Box<TagsWorld>),
    Culture(Box<CultureWorld>),
    Classes(Box<ClassesWorld>),
    Ethno(Box<EthnoWorld>),
    Opinions(Box<OpinionsWorld>),
    Structure(Box<StructureWorld>),
    Dpd(Box<DpdWorld>),
    Norms(Box<NormsWorld>),
    Agreement(Box<AgreementWorld>),
    Image(Box<ImageWorld>),
    Farol(Box<FarolWorld>),
    Ants(Box<AntsWorld>),
    Thresholds(Box<ThresholdsWorld>),
    Retirement(Box<RetirementWorld>),
    Punishment(Box<PunishmentWorld>),
    Zi(Box<ZiWorld>),
    Bali(Box<BaliWorld>),
    Line(Box<LineWorld>),
    Tipping(Box<TippingWorld>),
    Hoard(Box<HoardWorld>),
    Firms(Box<FirmsWorld>),
    Collusion(Box<CollusionWorld>),
    Auctions(Box<AuctionsWorld>),
    Polarity(Box<PolarityWorld>),
    Geosim(Box<GeosimWorld>),
    DemocraticPeace(Box<DemocraticPeaceWorld>),
}

impl ModelWorld {
    pub fn new(config: ModelConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        Self::with_landscapes(config, seed, &[])
    }

    /// Like `new`; `landscapes` are the sugarscape's painted maps
    /// (`World::with_landscapes`). Other models have no landscapes and ignore them.
    pub fn with_landscapes(
        config: ModelConfig,
        seed: u64,
        landscapes: &[Option<Vec<f64>>],
    ) -> Result<Self, Vec<FieldError>> {
        Ok(match config {
            ModelConfig::Sugarscape(c) => {
                ModelWorld::Sugarscape(Box::new(World::with_landscapes(c, seed, landscapes)?))
            }
            ModelConfig::Schelling(c) => {
                ModelWorld::Schelling(Box::new(SchellingWorld::new(c, seed)?))
            }
            ModelConfig::Ring(c) => ModelWorld::Ring(Box::new(RingWorld::new(c, seed)?)),
            ModelConfig::Anasazi(c) => ModelWorld::Anasazi(Box::new(AnasaziWorld::new(c, seed)?)),
            ModelConfig::Civil(c) => ModelWorld::Civil(Box::new(CivilWorld::new(c, seed)?)),
            ModelConfig::Spatial(c) => ModelWorld::Spatial(Box::new(SpatialWorld::new(c, seed)?)),
            ModelConfig::Tags(c) => ModelWorld::Tags(Box::new(TagsWorld::new(c, seed)?)),
            ModelConfig::Culture(c) => ModelWorld::Culture(Box::new(CultureWorld::new(c, seed)?)),
            ModelConfig::Classes(c) => ModelWorld::Classes(Box::new(ClassesWorld::new(c, seed)?)),
            ModelConfig::Ethno(c) => ModelWorld::Ethno(Box::new(EthnoWorld::new(c, seed)?)),
            ModelConfig::Opinions(c) => {
                ModelWorld::Opinions(Box::new(OpinionsWorld::new(c, seed)?))
            }
            ModelConfig::Structure(c) => {
                ModelWorld::Structure(Box::new(StructureWorld::new(c, seed)?))
            }
            ModelConfig::Dpd(c) => ModelWorld::Dpd(Box::new(DpdWorld::new(c, seed)?)),
            ModelConfig::Norms(c) => ModelWorld::Norms(Box::new(NormsWorld::new(c, seed)?)),
            ModelConfig::Agreement(c) => {
                ModelWorld::Agreement(Box::new(AgreementWorld::new(c, seed)?))
            }
            ModelConfig::Image(c) => ModelWorld::Image(Box::new(ImageWorld::new(c, seed)?)),
            ModelConfig::Farol(c) => ModelWorld::Farol(Box::new(FarolWorld::new(c, seed)?)),
            ModelConfig::Ants(c) => ModelWorld::Ants(Box::new(AntsWorld::new(c, seed)?)),
            ModelConfig::Thresholds(c) => {
                ModelWorld::Thresholds(Box::new(ThresholdsWorld::new(c, seed)?))
            }
            ModelConfig::Retirement(c) => {
                ModelWorld::Retirement(Box::new(RetirementWorld::new(c, seed)?))
            }
            ModelConfig::Punishment(c) => {
                ModelWorld::Punishment(Box::new(PunishmentWorld::new(c, seed)?))
            }
            ModelConfig::Zi(c) => ModelWorld::Zi(Box::new(ZiWorld::new(c, seed)?)),
            ModelConfig::Bali(c) => ModelWorld::Bali(Box::new(BaliWorld::new(c, seed)?)),
            ModelConfig::Line(c) => ModelWorld::Line(Box::new(LineWorld::new(c, seed)?)),
            ModelConfig::Tipping(c) => ModelWorld::Tipping(Box::new(TippingWorld::new(c, seed)?)),
            ModelConfig::Hoard(c) => ModelWorld::Hoard(Box::new(HoardWorld::new(c, seed)?)),
            ModelConfig::Firms(c) => ModelWorld::Firms(Box::new(FirmsWorld::new(c, seed)?)),
            ModelConfig::Polarity(c) => {
                ModelWorld::Polarity(Box::new(PolarityWorld::new(c, seed)?))
            }
            ModelConfig::Geosim(c) => ModelWorld::Geosim(Box::new(GeosimWorld::new(c, seed)?)),
            ModelConfig::DemocraticPeace(c) => {
                ModelWorld::DemocraticPeace(Box::new(DemocraticPeaceWorld::new(c, seed)?))
            }
            ModelConfig::Auctions(c) => {
                ModelWorld::Auctions(Box::new(AuctionsWorld::new(c, seed)?))
            }
            ModelConfig::Collusion(c) => {
                ModelWorld::Collusion(Box::new(CollusionWorld::new(c, seed)?))
            }
        })
    }

    pub fn kind(&self) -> ModelKind {
        match self {
            ModelWorld::Sugarscape(_) => ModelKind::Sugarscape,
            ModelWorld::Schelling(_) => ModelKind::Schelling,
            ModelWorld::Ring(_) => ModelKind::Ring,
            ModelWorld::Anasazi(_) => ModelKind::Anasazi,
            ModelWorld::Civil(_) => ModelKind::Civil,
            ModelWorld::Spatial(_) => ModelKind::Spatial,
            ModelWorld::Tags(_) => ModelKind::Tags,
            ModelWorld::Culture(_) => ModelKind::Culture,
            ModelWorld::Classes(_) => ModelKind::Classes,
            ModelWorld::Ethno(_) => ModelKind::Ethno,
            ModelWorld::Opinions(_) => ModelKind::Opinions,
            ModelWorld::Structure(_) => ModelKind::Structure,
            ModelWorld::Dpd(_) => ModelKind::Dpd,
            ModelWorld::Norms(_) => ModelKind::Norms,
            ModelWorld::Agreement(_) => ModelKind::Agreement,
            ModelWorld::Image(_) => ModelKind::Image,
            ModelWorld::Farol(_) => ModelKind::Farol,
            ModelWorld::Ants(_) => ModelKind::Ants,
            ModelWorld::Thresholds(_) => ModelKind::Thresholds,
            ModelWorld::Retirement(_) => ModelKind::Retirement,
            ModelWorld::Punishment(_) => ModelKind::Punishment,
            ModelWorld::Zi(_) => ModelKind::Zi,
            ModelWorld::Bali(_) => ModelKind::Bali,
            ModelWorld::Line(_) => ModelKind::Line,
            ModelWorld::Tipping(_) => ModelKind::Tipping,
            ModelWorld::Hoard(_) => ModelKind::Hoard,
            ModelWorld::Firms(_) => ModelKind::Firms,
            ModelWorld::Collusion(_) => ModelKind::Collusion,
            ModelWorld::Auctions(_) => ModelKind::Auctions,
            ModelWorld::Polarity(_) => ModelKind::Polarity,
            ModelWorld::Geosim(_) => ModelKind::Geosim,
            ModelWorld::DemocraticPeace(_) => ModelKind::DemocraticPeace,
        }
    }

    pub fn model(&self) -> &dyn Model {
        match self {
            ModelWorld::Sugarscape(w) => w.as_ref(),
            ModelWorld::Schelling(w) => w.as_ref(),
            ModelWorld::Ring(w) => w.as_ref(),
            ModelWorld::Anasazi(w) => w.as_ref(),
            ModelWorld::Civil(w) => w.as_ref(),
            ModelWorld::Spatial(w) => w.as_ref(),
            ModelWorld::Tags(w) => w.as_ref(),
            ModelWorld::Culture(w) => w.as_ref(),
            ModelWorld::Classes(w) => w.as_ref(),
            ModelWorld::Ethno(w) => w.as_ref(),
            ModelWorld::Opinions(w) => w.as_ref(),
            ModelWorld::Structure(w) => w.as_ref(),
            ModelWorld::Dpd(w) => w.as_ref(),
            ModelWorld::Norms(w) => w.as_ref(),
            ModelWorld::Agreement(w) => w.as_ref(),
            ModelWorld::Image(w) => w.as_ref(),
            ModelWorld::Farol(w) => w.as_ref(),
            ModelWorld::Ants(w) => w.as_ref(),
            ModelWorld::Thresholds(w) => w.as_ref(),
            ModelWorld::Retirement(w) => w.as_ref(),
            ModelWorld::Punishment(w) => w.as_ref(),
            ModelWorld::Zi(w) => w.as_ref(),
            ModelWorld::Bali(w) => w.as_ref(),
            ModelWorld::Line(w) => w.as_ref(),
            ModelWorld::Tipping(w) => w.as_ref(),
            ModelWorld::Hoard(w) => w.as_ref(),
            ModelWorld::Firms(w) => w.as_ref(),
            ModelWorld::Collusion(w) => w.as_ref(),
            ModelWorld::Auctions(w) => w.as_ref(),
            ModelWorld::Polarity(w) => w.as_ref(),
            ModelWorld::Geosim(w) => w.as_ref(),
            ModelWorld::DemocraticPeace(w) => w.as_ref(),
        }
    }

    pub fn model_mut(&mut self) -> &mut dyn Model {
        match self {
            ModelWorld::Sugarscape(w) => w.as_mut(),
            ModelWorld::Schelling(w) => w.as_mut(),
            ModelWorld::Ring(w) => w.as_mut(),
            ModelWorld::Anasazi(w) => w.as_mut(),
            ModelWorld::Civil(w) => w.as_mut(),
            ModelWorld::Spatial(w) => w.as_mut(),
            ModelWorld::Tags(w) => w.as_mut(),
            ModelWorld::Culture(w) => w.as_mut(),
            ModelWorld::Classes(w) => w.as_mut(),
            ModelWorld::Ethno(w) => w.as_mut(),
            ModelWorld::Opinions(w) => w.as_mut(),
            ModelWorld::Structure(w) => w.as_mut(),
            ModelWorld::Dpd(w) => w.as_mut(),
            ModelWorld::Norms(w) => w.as_mut(),
            ModelWorld::Agreement(w) => w.as_mut(),
            ModelWorld::Image(w) => w.as_mut(),
            ModelWorld::Farol(w) => w.as_mut(),
            ModelWorld::Ants(w) => w.as_mut(),
            ModelWorld::Thresholds(w) => w.as_mut(),
            ModelWorld::Retirement(w) => w.as_mut(),
            ModelWorld::Punishment(w) => w.as_mut(),
            ModelWorld::Zi(w) => w.as_mut(),
            ModelWorld::Bali(w) => w.as_mut(),
            ModelWorld::Line(w) => w.as_mut(),
            ModelWorld::Tipping(w) => w.as_mut(),
            ModelWorld::Hoard(w) => w.as_mut(),
            ModelWorld::Firms(w) => w.as_mut(),
            ModelWorld::Collusion(w) => w.as_mut(),
            ModelWorld::Auctions(w) => w.as_mut(),
            ModelWorld::Polarity(w) => w.as_mut(),
            ModelWorld::Geosim(w) => w.as_mut(),
            ModelWorld::DemocraticPeace(w) => w.as_mut(),
        }
    }

    pub fn sugarscape(&self) -> Option<&World> {
        match self {
            ModelWorld::Sugarscape(w) => Some(w),
            _ => None,
        }
    }

    pub fn sugarscape_mut(&mut self) -> Option<&mut World> {
        match self {
            ModelWorld::Sugarscape(w) => Some(w),
            _ => None,
        }
    }

    pub fn ring(&self) -> Option<&RingWorld> {
        match self {
            ModelWorld::Ring(w) => Some(w),
            _ => None,
        }
    }

    pub fn hoard(&self) -> Option<&HoardWorld> {
        match self {
            ModelWorld::Hoard(w) => Some(w),
            _ => None,
        }
    }

    pub fn anasazi(&self) -> Option<&AnasaziWorld> {
        match self {
            ModelWorld::Anasazi(w) => Some(w),
            _ => None,
        }
    }
}

/// A copy of a world's state without its statistics history (a keyframe):
/// about one copy of the current state, however long the run.
pub struct Checkpoint {
    world: ModelWorld,
    tick: u64,
}

impl Checkpoint {
    /// The tick the copy was taken at.
    pub fn tick(&self) -> u64 {
        self.tick
    }
}

/// Moves `$w`'s history out, clones it, and moves the history back.
macro_rules! copy_without_history {
    ($variant:ident, $w:expr) => {{
        let stats = std::mem::take(&mut $w.stats);
        let copy = (**$w).clone();
        $w.stats = stats;
        ModelWorld::$variant(Box::new(copy))
    }};
}

/// Replaces `$live` by a copy of `$kept`, giving it `$live`'s history cut to `$kept`'s tick.
macro_rules! restore_into {
    ($live:expr, $kept:expr) => {{
        let mut stats = std::mem::take(&mut $live.stats);
        stats.truncate($kept.tick as usize + 1);
        let mut next = (**$kept).clone();
        next.stats = stats;
        **$live = next;
    }};
}

impl ModelWorld {
    /// A keyframe of this world, or `None` for a model without them.
    #[allow(unreachable_patterns)]
    pub fn checkpoint(&mut self) -> Option<Checkpoint> {
        let tick = self.model().tick();
        let world = match self {
            ModelWorld::Sugarscape(w) => copy_without_history!(Sugarscape, w),
            ModelWorld::Schelling(w) => copy_without_history!(Schelling, w),
            ModelWorld::Ring(w) => copy_without_history!(Ring, w),
            ModelWorld::Anasazi(w) => copy_without_history!(Anasazi, w),
            ModelWorld::Civil(w) => copy_without_history!(Civil, w),
            ModelWorld::Spatial(w) => copy_without_history!(Spatial, w),
            ModelWorld::Tags(w) => copy_without_history!(Tags, w),
            ModelWorld::Culture(w) => copy_without_history!(Culture, w),
            ModelWorld::Classes(w) => copy_without_history!(Classes, w),
            ModelWorld::Ethno(w) => copy_without_history!(Ethno, w),
            ModelWorld::Opinions(w) => copy_without_history!(Opinions, w),
            ModelWorld::Structure(w) => copy_without_history!(Structure, w),
            ModelWorld::Dpd(w) => copy_without_history!(Dpd, w),
            ModelWorld::Norms(w) => copy_without_history!(Norms, w),
            ModelWorld::Agreement(w) => copy_without_history!(Agreement, w),
            // Without its private records, which the next generation rebuilds.
            ModelWorld::Image(w) => ModelWorld::Image(Box::new(w.keyframe())),
            ModelWorld::Farol(w) => copy_without_history!(Farol, w),
            ModelWorld::Ants(w) => copy_without_history!(Ants, w),
            ModelWorld::Thresholds(w) => copy_without_history!(Thresholds, w),
            ModelWorld::Retirement(w) => copy_without_history!(Retirement, w),
            ModelWorld::Punishment(w) => copy_without_history!(Punishment, w),
            ModelWorld::Zi(w) => copy_without_history!(Zi, w),
            ModelWorld::Bali(w) => copy_without_history!(Bali, w),
            ModelWorld::Line(w) => copy_without_history!(Line, w),
            ModelWorld::Tipping(w) => copy_without_history!(Tipping, w),
            ModelWorld::Hoard(w) => copy_without_history!(Hoard, w),
            ModelWorld::Firms(w) => copy_without_history!(Firms, w),
            ModelWorld::Collusion(w) => copy_without_history!(Collusion, w),
            ModelWorld::Polarity(w) => copy_without_history!(Polarity, w),
            ModelWorld::Geosim(w) => copy_without_history!(Geosim, w),
            ModelWorld::DemocraticPeace(w) => copy_without_history!(DemocraticPeace, w),
            ModelWorld::Auctions(w) => {
                let stats = std::mem::take(&mut w.stats);
                let mut copy = (**w).clone();
                if let Some(last) = stats.latest() {
                    copy.stats.push(last.clone());
                }
                w.stats = stats;
                ModelWorld::Auctions(Box::new(copy))
            }
            _ => return None,
        };
        Some(Checkpoint { world, tick })
    }

    /// Returns this world to `cp`, keeping its statistics history up to `cp`'s tick. The world
    /// must have reached that tick (its history must hold it) and be of the same model.
    #[allow(unreachable_patterns)]
    pub fn restore(&mut self, cp: &Checkpoint) -> Result<(), String> {
        if self.model().tick() < cp.tick {
            return Err(format!("this world has not reached tick {}", cp.tick));
        }
        match (self, &cp.world) {
            (ModelWorld::Sugarscape(live), ModelWorld::Sugarscape(kept)) => {
                restore_into!(live, kept)
            }
            (ModelWorld::Schelling(live), ModelWorld::Schelling(kept)) => restore_into!(live, kept),
            (ModelWorld::Ring(live), ModelWorld::Ring(kept)) => restore_into!(live, kept),
            (ModelWorld::Anasazi(live), ModelWorld::Anasazi(kept)) => restore_into!(live, kept),
            (ModelWorld::Civil(live), ModelWorld::Civil(kept)) => restore_into!(live, kept),
            (ModelWorld::Spatial(live), ModelWorld::Spatial(kept)) => restore_into!(live, kept),
            (ModelWorld::Tags(live), ModelWorld::Tags(kept)) => restore_into!(live, kept),
            (ModelWorld::Culture(live), ModelWorld::Culture(kept)) => restore_into!(live, kept),
            (ModelWorld::Classes(live), ModelWorld::Classes(kept)) => restore_into!(live, kept),
            (ModelWorld::Ethno(live), ModelWorld::Ethno(kept)) => restore_into!(live, kept),
            (ModelWorld::Opinions(live), ModelWorld::Opinions(kept)) => restore_into!(live, kept),
            (ModelWorld::Structure(live), ModelWorld::Structure(kept)) => restore_into!(live, kept),
            (ModelWorld::Dpd(live), ModelWorld::Dpd(kept)) => restore_into!(live, kept),
            (ModelWorld::Norms(live), ModelWorld::Norms(kept)) => restore_into!(live, kept),
            (ModelWorld::Agreement(live), ModelWorld::Agreement(kept)) => {
                restore_into!(live, kept)
            }
            (ModelWorld::Image(live), ModelWorld::Image(kept)) => restore_into!(live, kept),
            (ModelWorld::Farol(live), ModelWorld::Farol(kept)) => restore_into!(live, kept),
            (ModelWorld::Ants(live), ModelWorld::Ants(kept)) => restore_into!(live, kept),
            (ModelWorld::Thresholds(live), ModelWorld::Thresholds(kept)) => {
                restore_into!(live, kept)
            }
            (ModelWorld::Retirement(live), ModelWorld::Retirement(kept)) => {
                restore_into!(live, kept)
            }
            (ModelWorld::Punishment(live), ModelWorld::Punishment(kept)) => {
                restore_into!(live, kept)
            }
            (ModelWorld::Zi(live), ModelWorld::Zi(kept)) => {
                restore_into!(live, kept)
            }
            (ModelWorld::Bali(live), ModelWorld::Bali(kept)) => {
                restore_into!(live, kept)
            }
            (ModelWorld::Line(live), ModelWorld::Line(kept)) => restore_into!(live, kept),
            (ModelWorld::Tipping(live), ModelWorld::Tipping(kept)) => restore_into!(live, kept),
            (ModelWorld::Hoard(live), ModelWorld::Hoard(kept)) => restore_into!(live, kept),
            (ModelWorld::Firms(live), ModelWorld::Firms(kept)) => restore_into!(live, kept),
            (ModelWorld::Collusion(live), ModelWorld::Collusion(kept)) => restore_into!(live, kept),
            (ModelWorld::Polarity(live), ModelWorld::Polarity(kept)) => restore_into!(live, kept),
            (ModelWorld::Geosim(live), ModelWorld::Geosim(kept)) => restore_into!(live, kept),
            (ModelWorld::DemocraticPeace(live), ModelWorld::DemocraticPeace(kept)) => {
                restore_into!(live, kept)
            }
            (ModelWorld::Auctions(live), ModelWorld::Auctions(kept)) => {
                let mut stats = std::mem::take(&mut live.stats);
                let len = stats.history().partition_point(|s| s.tick <= kept.tick);
                stats.truncate(len);
                if stats.latest().is_none_or(|s| s.tick != kept.tick) {
                    if let Some(last) = kept.stats.latest() {
                        stats.push(last.clone());
                    }
                }
                let mut next = (**kept).clone();
                next.stats = stats;
                **live = next;
            }
            _ => return Err("the keyframe is of another model".into()),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presets;
    use serde_json::json;

    #[test]
    fn untagged_and_sugarscape_tagged_configs_are_sugarscape() {
        let plain = ModelConfig::from_json(r#"{"population": 100}"#).unwrap();
        let tagged =
            ModelConfig::from_json(r#"{"model": "sugarscape", "population": 100}"#).unwrap();
        assert_eq!(plain, tagged);
        assert_eq!(plain.kind(), ModelKind::Sugarscape);
        assert_eq!(plain.sugarscape().unwrap().population, 100);
    }

    #[test]
    fn sugarscape_configs_serialize_without_a_tag() {
        let c = presets::by_id("iv-3-trade").unwrap().config;
        let model = ModelConfig::from(c.clone());
        assert_eq!(
            serde_json::to_string(&model).unwrap(),
            serde_json::to_string(&c).unwrap()
        );
        assert!(!serde_json::to_string(&model).unwrap().contains("\"model\""));
    }

    #[test]
    fn schelling_configs_round_trip_with_their_tag() {
        let c = ModelConfig::from_json(r#"{"model": "schelling", "population": 100}"#).unwrap();
        assert_eq!(c.kind(), ModelKind::Schelling);
        let ModelConfig::Schelling(s) = &c else {
            unreachable!()
        };
        assert_eq!(
            (s.population, s.width),
            (100, 16),
            "missing fields take the defaults (Schelling's 1971 board)"
        );
        let json = serde_json::to_value(&c).unwrap();
        assert_eq!(json["model"], "schelling");
        assert_eq!(ModelConfig::from_value(json).unwrap(), c);
        let e = ModelConfig::from_json(r#"{"model": "schelling", "vision": 3}"#).unwrap_err();
        assert!(e[0].message.contains("vision"), "{e:?}");
        let e =
            ModelConfig::from_json(r#"{"model": "schelling", "population": 2500}"#).unwrap_err();
        assert_eq!(e[0].field, "population");
    }

    #[test]
    fn with_path_sets_another_models_fields() {
        let c = ModelConfig::Schelling(SchellingConfig::default());
        let next = c.with_path("preference.max", &json!(0.75)).unwrap();
        let ModelConfig::Schelling(s) = &next else {
            unreachable!()
        };
        assert_eq!((s.preference.min, s.preference.max), (0.5, 0.75));
        assert_eq!(
            c.with_path("vision.max", &json!(3)).unwrap_err().message,
            "unknown field vision.max"
        );
        assert!(c.with_path("model", &json!("sugarscape")).is_err());
        assert!(c.with_path("population", &json!("many")).is_err());
    }

    #[test]
    fn a_world_refuses_another_models_config() {
        let mut any = ModelWorld::new(ModelConfig::from(Config::default()), 1).unwrap();
        let e = any
            .model_mut()
            .set_config(ModelConfig::Schelling(SchellingConfig::default()))
            .unwrap_err();
        assert_eq!(e[0].field, "model");
        let mut s = ModelWorld::new(ModelConfig::Schelling(SchellingConfig::default()), 1).unwrap();
        assert!(s.model_mut().set_config(Config::default().into()).is_err());
        assert!(s.sugarscape().is_none());
    }

    #[test]
    fn ring_configs_round_trip_with_their_tag() {
        let c = ModelConfig::from_json(r#"{"model": "ring", "start": "megagroup"}"#).unwrap();
        assert_eq!(c.kind(), ModelKind::Ring);
        let json = serde_json::to_value(&c).unwrap();
        assert_eq!(
            (json["model"].as_str(), json["sites"].as_u64()),
            (Some("ring"), Some(150))
        );
        assert_eq!(ModelConfig::from_value(json).unwrap(), c);
        assert_eq!(
            c.series_names(),
            [
                "flocks",
                "mean_flock",
                "largest_flock",
                "mean_distance",
                "population"
            ]
        );
        let e = ModelConfig::from_json(r#"{"model": "ring", "start": "clumps"}"#).unwrap_err();
        assert_eq!(e[0].field, "config");
    }

    #[test]
    fn anasazi_configs_round_trip_with_their_tag() {
        let c = ModelConfig::from_json(r#"{"model": "anasazi", "quirks": {"wrap_edges": false}}"#)
            .unwrap();
        assert_eq!(c.kind(), ModelKind::Anasazi);
        let ModelConfig::Anasazi(a) = &c else {
            unreachable!()
        };
        assert!(
            !a.quirks.wrap_edges && a.quirks.occupancy_leak,
            "missing fields take the defaults"
        );
        assert_eq!(a.harvest_adjustment, 0.56);
        let json = serde_json::to_value(&c).unwrap();
        assert_eq!(json["model"], "anasazi");
        assert_eq!(ModelConfig::from_value(json).unwrap(), c);
        assert_eq!(c.series_names()[..3], ["households", "historical", "fit"]);
        let next = c.with_path("quirks.occupancy_leak", &json!(false)).unwrap();
        let ModelConfig::Anasazi(n) = &next else {
            unreachable!()
        };
        assert!(!n.quirks.occupancy_leak);
        let e = ModelConfig::from_json(r#"{"model": "anasazi", "end_year": 700}"#).unwrap_err();
        assert_eq!(e[0].field, "end_year");
        let w = ModelWorld::new(c, 1).unwrap();
        assert!(w.anasazi().is_some() && w.sugarscape().is_none());
        assert_eq!(w.model().size(), (80, 120));
    }

    #[test]
    fn civil_configs_round_trip_with_their_tag() {
        let c = ModelConfig::from_json(
            r#"{"model": "civil", "variant": "ethnic", "quirks": {"floor_ratio": true}}"#,
        )
        .unwrap();
        assert_eq!(c.kind(), ModelKind::Civil);
        let ModelConfig::Civil(v) = &c else {
            unreachable!()
        };
        assert!(v.quirks.floor_ratio && !v.quirks.jailed_stay);
        assert_eq!(
            (v.width, v.legitimacy),
            (40, 0.82),
            "missing fields take the defaults"
        );
        let json = serde_json::to_value(&c).unwrap();
        assert_eq!(json["model"], "civil");
        assert_eq!(ModelConfig::from_value(json).unwrap(), c);
        assert_eq!(c.series_names()[..3], ["population", "active", "quiet"]);
        assert_eq!(c.max_ticks(), None);
        let next = c.with_path("vision.cop", &json!(3.0)).unwrap();
        let ModelConfig::Civil(n) = &next else {
            unreachable!()
        };
        assert_eq!(n.vision.cop, 3.0);
        let e = ModelConfig::from_json(r#"{"model": "civil", "legitimacy": 2}"#).unwrap_err();
        assert_eq!(e[0].field, "legitimacy");
        let w = ModelWorld::new(c, 1).unwrap();
        assert_eq!(w.kind(), ModelKind::Civil);
        assert_eq!(w.model().size(), (40, 40));
    }

    #[test]
    fn spatial_configs_round_trip_with_their_tag() {
        let c = ModelConfig::from_json(
            r#"{"model": "spatial", "lattice": "cube", "width": 10, "update": "asynchronous"}"#,
        )
        .unwrap();
        assert_eq!(c.kind(), ModelKind::Spatial);
        let json = serde_json::to_value(&c).unwrap();
        assert_eq!(json["model"], "spatial");
        assert_eq!(ModelConfig::from_value(json).unwrap(), c);
        assert_eq!(c.series_names()[0], "fraction_c");
        let next = c.with_path("b", &json!(1.6)).unwrap();
        let ModelConfig::Spatial(s) = &next else {
            unreachable!()
        };
        assert_eq!(s.b, 1.6);
        let e = ModelConfig::from_json(r#"{"model": "spatial", "b": 0}"#).unwrap_err();
        assert_eq!(e[0].field, "b");
        let mut w = ModelWorld::new(c, 1).unwrap();
        assert_eq!((w.kind(), w.model().size()), (ModelKind::Spatial, (10, 10)));
        assert_eq!(w.model().population(), 1000);
        let cp = w.checkpoint().expect("spatial worlds have keyframes");
        w.model_mut().run(3);
        w.restore(&cp).unwrap();
        assert_eq!(w.model().tick(), 0);
    }

    #[test]
    fn tags_configs_round_trip_with_their_tag() {
        let c = ModelConfig::from_json(r#"{"model": "tags", "tie_rule": "current"}"#).unwrap();
        assert_eq!(c.kind(), ModelKind::Tags);
        let ModelConfig::Tags(t) = &c else {
            unreachable!()
        };
        assert_eq!(t.tie_rule, crate::tags::TieRule::Current);
        assert_eq!(
            (t.agents, t.pairings),
            (100, 3),
            "missing fields take the defaults"
        );
        let json = serde_json::to_value(&c).unwrap();
        assert_eq!(json["model"], "tags");
        assert_eq!(ModelConfig::from_value(json).unwrap(), c);
        assert_eq!(c.series_names()[..2], ["donation_rate", "mean_tolerance"]);
        assert_eq!(c.max_ticks(), Some(30_000));
        let forever = c.with_path("end", &json!(0)).unwrap();
        assert_eq!(forever.max_ticks(), None);
        let e = ModelConfig::from_json(r#"{"model": "tags", "cost": -1}"#).unwrap_err();
        assert_eq!(e[0].field, "cost");
        let w = ModelWorld::new(c, 1).unwrap();
        assert_eq!(w.kind(), ModelKind::Tags);
        assert_eq!(w.model().size(), (100, 200));
    }

    #[test]
    fn ethno_configs_round_trip_with_their_tag() {
        let c = ModelConfig::from_json(
            r#"{"model": "ethno", "colors": 5, "allowed": ["H", "S", "T"]}"#,
        )
        .unwrap();
        assert_eq!(c.kind(), ModelKind::Ethno);
        let json = serde_json::to_value(&c).unwrap();
        assert_eq!(json["model"], "ethno");
        assert_eq!(json["tag_mutation"], serde_json::Value::Null);
        assert_eq!(ModelConfig::from_value(json).unwrap(), c);
        assert_eq!(c.series_names()[..2], ["population", "ethnocentric"]);
        assert_eq!(c.max_ticks(), Some(2000));
        let next = c.with_path("tag_mutation", &json!(0.3)).unwrap();
        let ModelConfig::Ethno(e) = &next else {
            unreachable!()
        };
        assert_eq!(e.tag_mutation, Some(0.3));
        let e = ModelConfig::from_json(r#"{"model": "ethno", "cost": -1}"#).unwrap_err();
        assert_eq!(e[0].field, "cost");
        let mut w = ModelWorld::new(c, 1).unwrap();
        assert_eq!((w.kind(), w.model().size()), (ModelKind::Ethno, (50, 50)));
        let cp = w.checkpoint().expect("ethno worlds have keyframes");
        w.model_mut().run(3);
        w.restore(&cp).unwrap();
        assert_eq!(w.model().tick(), 0);
    }

    #[test]
    fn dpd_configs_round_trip_with_their_tag() {
        let c = ModelConfig::from_json(
            r#"{"model": "dpd", "max_age": 100, "removal": "end_of_cycle"}"#,
        )
        .unwrap();
        assert_eq!(c.kind(), ModelKind::Dpd);
        let json = serde_json::to_value(&c).unwrap();
        assert_eq!(json["model"], "dpd");
        assert_eq!(json["removal"], "end_of_cycle");
        assert_eq!(ModelConfig::from_value(json).unwrap(), c);
        assert_eq!(c.series_names()[..2], ["cooperators", "defectors"]);
        assert_eq!(c.max_ticks(), None);
        let next = c.with_path("end", &json!(500)).unwrap();
        assert_eq!(next.max_ticks(), Some(500));
        let e = ModelConfig::from_json(r#"{"model": "dpd", "mutation": 2}"#).unwrap_err();
        assert_eq!(e[0].field, "mutation");
        let mut w = ModelWorld::new(c, 1).unwrap();
        assert_eq!((w.kind(), w.model().size()), (ModelKind::Dpd, (30, 30)));
        assert_eq!(w.model().population(), 100);
        let cp = w.checkpoint().expect("dpd worlds have keyframes");
        w.model_mut().run(3);
        w.restore(&cp).unwrap();
        assert_eq!(w.model().tick(), 0);
    }

    #[test]
    fn image_configs_round_trip_with_their_tag() {
        let c = ModelConfig::from_json(
            r#"{"model": "image", "rounds": 300, "strategies": ["and"], "offset": "none"}"#,
        )
        .unwrap();
        assert_eq!(c.kind(), ModelKind::Image);
        let json = serde_json::to_value(&c).unwrap();
        assert_eq!(json["model"], "image");
        assert_eq!(json["offset"], "none");
        assert_eq!(ModelConfig::from_value(json).unwrap(), c);
        assert_eq!(c.series_names()[..2], ["help_rate", "mean_k"]);
        assert_eq!(c.max_ticks(), None);
        let next = c.with_path("end", &json!(500)).unwrap();
        assert_eq!(next.max_ticks(), Some(500));
        let e = ModelConfig::from_json(r#"{"model": "image", "mutation": 2}"#).unwrap_err();
        assert_eq!(e[0].field, "mutation");
        let mut w = ModelWorld::new(c, 1).unwrap();
        assert_eq!((w.kind(), w.model().size()), (ModelKind::Image, (10, 10)));
        assert_eq!(w.model().population(), 100);
        let cp = w.checkpoint().expect("image worlds have keyframes");
        w.model_mut().run(3);
        w.restore(&cp).unwrap();
        assert_eq!(w.model().tick(), 0);
    }

    #[test]
    fn farol_configs_round_trip_with_their_tag() {
        let c = ModelConfig::from_json(
            r#"{"model": "farol", "game": "minority", "agents": 101, "capacity": 50, "memory": 6}"#,
        )
        .unwrap();
        assert_eq!(c.kind(), ModelKind::Farol);
        let json = serde_json::to_value(&c).unwrap();
        assert_eq!(
            (json["model"].as_str(), json["strategies"].as_u64()),
            (Some("farol"), Some(12))
        );
        assert_eq!(ModelConfig::from_value(json).unwrap(), c);
        assert_eq!(c.series_names()[..2], ["attendance", "crowded"]);
        let e = ModelConfig::from_json(r#"{"model": "farol", "capacity": 500}"#).unwrap_err();
        assert_eq!(e[0].field, "capacity");
        let mut w = ModelWorld::new(c, 1).unwrap();
        assert_eq!(w.kind(), ModelKind::Farol);
        let cp = w.checkpoint().expect("farol worlds have keyframes");
        w.model_mut().run(3);
        w.restore(&cp).unwrap();
        assert_eq!(w.model().tick(), 0);
    }

    #[test]
    fn ants_configs_round_trip_with_their_tag() {
        let c = ModelConfig::from_json(
            r#"{"model": "ants", "ants": 200, "rule": "alfarano", "network": "random", "a": 0.05}"#,
        )
        .unwrap();
        assert_eq!(c.kind(), ModelKind::Ants);
        let json = serde_json::to_value(&c).unwrap();
        assert_eq!(
            (json["model"].as_str(), json["meetings"].as_u64()),
            (Some("ants"), Some(50))
        );
        assert_eq!(ModelConfig::from_value(json).unwrap(), c);
        assert_eq!(c.series_names()[..2], ["share", "top_share"]);
        let e = ModelConfig::from_json(r#"{"model": "ants", "sources": 9}"#).unwrap_err();
        assert_eq!(e[0].field, "sources");
        let mut w = ModelWorld::new(c, 1).unwrap();
        assert_eq!(w.kind(), ModelKind::Ants);
        let cp = w.checkpoint().expect("ants worlds have keyframes");
        w.model_mut().run(3);
        w.restore(&cp).unwrap();
        assert_eq!(w.model().tick(), 0);
    }

    #[test]
    fn thresholds_configs_round_trip_with_their_tag() {
        let c = ModelConfig::from_json(
            r#"{"model": "thresholds", "distribution": "normal", "sd": 0.13, "friends": {"enabled": true}}"#,
        )
        .unwrap();
        assert_eq!(c.kind(), ModelKind::Thresholds);
        let json = serde_json::to_value(&c).unwrap();
        assert_eq!(
            (json["model"].as_str(), json["actors"].as_u64()),
            (Some("thresholds"), Some(100))
        );
        assert_eq!(ModelConfig::from_value(json).unwrap(), c);
        assert_eq!(c.series_names()[..2], ["acting", "step"]);
        let e = ModelConfig::from_json(r#"{"model": "thresholds", "trigger": "hub"}"#).unwrap_err();
        assert_eq!(e[0].field, "trigger");
        let mut w = ModelWorld::new(c, 1).unwrap();
        assert_eq!(w.kind(), ModelKind::Thresholds);
        let cp = w.checkpoint().expect("thresholds worlds have keyframes");
        w.model_mut().run(3);
        w.restore(&cp).unwrap();
        assert_eq!(w.model().tick(), 0);
    }

    #[test]
    fn retirement_configs_round_trip_with_their_tag() {
        let c = ModelConfig::from_json(
            r#"{"model": "retirement", "per_cohort": 20, "renewal": "replace", "size": {"min": 5, "max": 9}}"#,
        )
        .unwrap();
        assert_eq!(c.kind(), ModelKind::Retirement);
        let json = serde_json::to_value(&c).unwrap();
        assert_eq!(
            (json["model"].as_str(), json["extent"].as_u64()),
            (Some("retirement"), Some(5))
        );
        assert_eq!(ModelConfig::from_value(json).unwrap(), c);
        assert_eq!(c.series_names()[..2], ["retired", "retired_a"]);
        let e = ModelConfig::from_json(r#"{"model": "retirement", "norm": 0}"#).unwrap_err();
        assert_eq!(e[0].field, "norm");
        let mut w = ModelWorld::new(c, 1).unwrap();
        assert_eq!(w.kind(), ModelKind::Retirement);
        let cp = w.checkpoint().expect("retirement worlds have keyframes");
        w.model_mut().run(3);
        w.restore(&cp).unwrap();
        assert_eq!(w.model().tick(), 0);
    }

    #[test]
    fn punishment_configs_round_trip_with_their_tag() {
        let c = ModelConfig::from_json(
            r#"{"model": "punishment", "size": 8, "groups": 16, "victory": "tanh", "erring": "self"}"#,
        )
        .unwrap();
        assert_eq!(c.kind(), ModelKind::Punishment);
        let json = serde_json::to_value(&c).unwrap();
        assert_eq!(
            (json["model"].as_str(), json["fine"].as_f64()),
            (Some("punishment"), Some(0.8))
        );
        assert_eq!(ModelConfig::from_value(json).unwrap(), c);
        assert_eq!(c.series_names()[..2], ["cooperation", "contributors"]);
        let e = ModelConfig::from_json(r#"{"model": "punishment", "error": 2}"#).unwrap_err();
        assert_eq!(e[0].field, "error");
        let mut w = ModelWorld::new(c, 1).unwrap();
        assert_eq!(w.kind(), ModelKind::Punishment);
        let cp = w.checkpoint().expect("punishment worlds have keyframes");
        w.model_mut().run(3);
        w.restore(&cp).unwrap();
        assert_eq!(w.model().tick(), 0);
    }

    #[test]
    fn firms_configs_round_trip_with_their_tag() {
        let c = ModelConfig::from_json(
            r#"{"model": "firms", "agents": 50, "beta": 1.8, "stop_at": 3}"#,
        )
        .unwrap();
        assert_eq!(c.kind(), ModelKind::Firms);
        let json = serde_json::to_value(&c).unwrap();
        assert_eq!(
            (json["model"].as_str(), json["neighbors"].as_u64()),
            (Some("firms"), Some(2))
        );
        assert_eq!(ModelConfig::from_value(json).unwrap(), c);
        assert_eq!(c.series_names()[..2], ["firms", "births"]);
        let e = ModelConfig::from_json(r#"{"model": "firms", "agents": 1}"#).unwrap_err();
        assert_eq!(e[0].field, "agents");
        let mut w = ModelWorld::new(c, 1).unwrap();
        assert_eq!(w.kind(), ModelKind::Firms);
        let cp = w.checkpoint().expect("firms worlds have keyframes");
        w.model_mut().run(3);
        w.restore(&cp).unwrap();
        assert_eq!(w.model().tick(), 0);
    }

    #[test]
    fn polarity_host_runs_partial_final_tick_and_preserves_endpoint() {
        let config = ModelConfig::from_json(r#"{"model":"polarity","width":2,"height":2,"predator_share":0,"horizon":15,"periods_per_tick":7}"#).unwrap();
        assert_eq!(config.max_ticks(), Some(3));
        let mut world = ModelWorld::new(config.clone(), 1).unwrap();
        world.model_mut().run(3);
        let latest: serde_json::Value = serde_json::from_str(&world.model().latest_json()).unwrap();
        assert_eq!(
            (
                latest["periods"].as_u64(),
                latest["last_tick_periods"].as_u64()
            ),
            (Some(15), Some(1))
        );
        assert_eq!(world.model().latest_value("sovereign_count"), Some(4.0));
        let fingerprint = world.model().fingerprint();
        world.model_mut().run(10);
        assert_eq!(world.model().fingerprint(), fingerprint);
        assert!(world
            .model_mut()
            .set_config(config.with_path("alliances", &json!(true)).unwrap())
            .is_err());
    }

    #[test]
    fn auctions_checkpoint_restores_latest_snapshot_across_truncated_history_gap() {
        let config = ModelConfig::Auctions(crate::auctions::AuctionsConfig {
            horizon: 23,
            window: 5,
            periods_per_tick: 1,
            ..Default::default()
        });
        let mut w = ModelWorld::new(config, 1).unwrap();
        w.model_mut().run(4);
        let checkpoint = w.checkpoint().unwrap();
        w.model_mut().run(3);
        let expected = w.model().fingerprint();
        let ModelWorld::Auctions(a) = &mut w else {
            unreachable!()
        };
        let latest = a.stats.latest().cloned().unwrap();
        a.stats.truncate(2);
        a.stats.push(latest);
        w.restore(&checkpoint).unwrap();
        let ModelWorld::Auctions(a) = &w else {
            unreachable!()
        };
        assert_eq!(
            a.stats.history().iter().map(|s| s.tick).collect::<Vec<_>>(),
            vec![0, 1, 4]
        );
        w.model_mut().run(3);
        assert_eq!(w.model().fingerprint(), expected);
    }
    #[test]
    fn auctions_configs_round_trip_and_checkpoint_restores_random_stream() {
        let config = ModelConfig::from_json(
            r#"{"model":"auctions","horizon":23,"window":5,"periods_per_tick":7}"#,
        )
        .unwrap();
        assert_eq!(config.kind(), ModelKind::Auctions);
        assert_eq!(
            ModelConfig::from_json(&serde_json::to_string(&config).unwrap()).unwrap(),
            config
        );
        let mut w = ModelWorld::new(config, 1).unwrap();
        w.model_mut().run(1);
        let checkpoint = w.checkpoint().unwrap();
        w.model_mut().run(1);
        let expected = w.model().fingerprint();
        w.restore(&checkpoint).unwrap();
        w.model_mut().run(1);
        assert_eq!(w.model().fingerprint(), expected);
    }
    #[test]
    fn collusion_configs_round_trip_with_their_tag() {
        let c = ModelConfig::from_json(
            r#"{"model": "collusion", "memory": 0, "delta": 0.5, "window": 50}"#,
        )
        .unwrap();
        assert_eq!(c.kind(), ModelKind::Collusion);
        let json = serde_json::to_value(&c).unwrap();
        assert_eq!(
            (json["model"].as_str(), json["prices"].as_u64()),
            (Some("collusion"), Some(15))
        );
        assert_eq!(ModelConfig::from_value(json).unwrap(), c);
        assert_eq!(c.series_names()[..2], ["price_1", "price_2"]);
        let e = ModelConfig::from_json(r#"{"model": "collusion", "firms": 9}"#).unwrap_err();
        assert_eq!(e[0].field, "firms");
        let mut w = ModelWorld::new(c, 1).unwrap();
        assert_eq!(w.kind(), ModelKind::Collusion);
        let cp = w.checkpoint().expect("collusion worlds have keyframes");
        w.model_mut().run(3);
        w.restore(&cp).unwrap();
        assert_eq!(w.model().tick(), 0);
    }

    #[test]
    fn bali_configs_round_trip_with_their_tag() {
        let c = ModelConfig::from_json(
            r#"{"model": "bali", "plans": "traditional", "growth": 2.4, "stop_at": 2}"#,
        )
        .unwrap();
        assert_eq!(c.kind(), ModelKind::Bali);
        let json = serde_json::to_value(&c).unwrap();
        assert_eq!(
            (json["model"].as_str(), json["level"].as_u64()),
            (Some("bali"), Some(14))
        );
        assert_eq!(ModelConfig::from_value(json).unwrap(), c);
        assert_eq!(c.series_names()[..2], ["harvest", "spread"]);
        let e = ModelConfig::from_json(r#"{"model": "bali", "growth": -1}"#).unwrap_err();
        assert_eq!(e[0].field, "growth");
        let mut w = ModelWorld::new(c, 1).unwrap();
        assert_eq!(w.kind(), ModelKind::Bali);
        let cp = w.checkpoint().expect("bali worlds have keyframes");
        w.model_mut().run(3);
        w.restore(&cp).unwrap();
        assert_eq!(w.model().tick(), 0);
    }

    #[test]
    fn zi_configs_round_trip_with_their_tag() {
        let c = ModelConfig::from_json(
            r#"{"model": "zi", "market": "gs4", "strategy": "zi_u", "shouts": 500}"#,
        )
        .unwrap();
        assert_eq!(c.kind(), ModelKind::Zi);
        let json = serde_json::to_value(&c).unwrap();
        assert_eq!(
            (json["model"].as_str(), json["price_max"].as_u64()),
            (Some("zi"), Some(200))
        );
        assert_eq!(ModelConfig::from_value(json).unwrap(), c);
        assert_eq!(c.series_names()[..2], ["price", "mean_price"]);
        let e = ModelConfig::from_json(r#"{"model": "zi", "shouts": 0}"#).unwrap_err();
        assert_eq!(e[0].field, "shouts");
        let mut w = ModelWorld::new(c, 1).unwrap();
        assert_eq!(w.kind(), ModelKind::Zi);
        let cp = w.checkpoint().expect("zi worlds have keyframes");
        w.model_mut().run(3);
        w.restore(&cp).unwrap();
        assert_eq!(w.model().tick(), 0);
    }

    #[test]
    fn hoard_configs_round_trip_with_their_tag() {
        let c = ModelConfig::from_json(r#"{"model": "hoard", "app_lard": 3.0, "generations": 5}"#)
            .unwrap();
        assert_eq!(c.kind(), ModelKind::Hoard);
        let json = serde_json::to_value(&c).unwrap();
        assert_eq!(
            (json["model"].as_str(), json["n"].as_u64()),
            (Some("hoard"), Some(20))
        );
        assert_eq!(ModelConfig::from_value(json).unwrap(), c);
        assert_eq!(c.series_names()[..2], ["generation", "mean_larder_prob"]);
        let e = ModelConfig::from_json(r#"{"model": "hoard", "n": 1}"#).unwrap_err();
        assert_eq!(e[0].field, "n");
        let mut w = ModelWorld::new(c, 1).unwrap();
        assert_eq!(w.kind(), ModelKind::Hoard);
        let cp = w.checkpoint().expect("hoard worlds have keyframes");
        w.model_mut().run(3);
        w.restore(&cp).unwrap();
        assert_eq!(w.model().tick(), 0);
    }

    #[test]
    fn only_the_anasazi_finishes() {
        let mut w = ModelWorld::new(
            ModelConfig::Anasazi(crate::anasazi::AnasaziConfig {
                start_year: 1349,
                ..Default::default()
            }),
            1,
        )
        .unwrap();
        assert!(!w.model().finished());
        w.model_mut().run(5);
        assert!(w.model().finished());
        assert_eq!(w.model().tick(), 1);
        let s = ModelWorld::new(ModelConfig::Ring(RingConfig::default()), 1).unwrap();
        assert!(!s.model().finished());
    }

    #[test]
    fn every_kind_names_itself_and_only_other_models_have_schemas() {
        let names: Vec<&str> = ModelKind::ALL.iter().map(|k| k.as_str()).collect();
        assert_eq!(
            names,
            [
                "sugarscape",
                "schelling",
                "ring",
                "anasazi",
                "civil",
                "spatial",
                "tags",
                "culture",
                "classes",
                "ethno",
                "opinions",
                "structure",
                "dpd",
                "norms",
                "agreement",
                "image",
                "farol",
                "ants",
                "thresholds",
                "retirement",
                "punishment",
                "zi",
                "bali",
                "line",
                "tipping",
                "hoard",
                "firms",
                "collusion",
                "auctions",
                "polarity",
                "geosim",
                "democratic_peace"
            ]
        );
        assert!(ModelKind::Sugarscape.schema().is_empty());
        for kind in &ModelKind::ALL[1..] {
            assert!(!kind.schema().is_empty(), "{kind:?}");
        }
    }

    #[test]
    fn unknown_models_are_field_errors() {
        let e = ModelConfig::from_json(r#"{"model": "boids"}"#).unwrap_err();
        assert_eq!(e[0].field, "model");
        assert!(e[0].message.contains("\"boids\""), "{e:?}");
        let e = ModelConfig::from_json(r#"{"model": 3}"#).unwrap_err();
        assert_eq!(e[0].field, "model");
    }

    #[test]
    fn with_path_cannot_change_the_model() {
        let c = ModelConfig::from(Config::default());
        assert!(c.with_path("model", &json!("schelling")).is_err());
        let next = c.with_path("population", &json!(10)).unwrap();
        assert_eq!(next.sugarscape().unwrap().population, 10);
    }

    #[test]
    fn a_sugarscape_model_world_is_the_world() {
        let config = presets::by_id("ii-2-unit").unwrap().config;
        let mut direct = World::new(config.clone(), 1).unwrap();
        let mut any = ModelWorld::new(config.into(), 1).unwrap();
        direct.run(50);
        any.model_mut().run(50);
        let m = any.model();
        assert_eq!(m.fingerprint(), direct.fingerprint());
        assert_eq!((m.tick(), m.population()), (50, direct.population()));
        assert_eq!(m.series("population"), direct.stats.series("population"));
        assert_eq!(m.series_csv(), export::series_csv(&direct));
        assert_eq!(m.size(), (50, 50));
        assert!(m.render("nope", "resource:0", &mut Vec::new()).is_err());
        assert!(any.sugarscape().is_some());
    }

    #[test]
    fn a_sugarscape_world_refuses_structural_changes() {
        let mut c = Config::default();
        c.goods[0].map = crate::config::Map::Flat { capacity: 4.0 };
        let mut any = ModelWorld::new(c.clone().into(), 1).unwrap();
        c.width = 60;
        let e = any.model_mut().set_config(c.into()).unwrap_err();
        assert_eq!(e[0].field, "width");
    }
}
