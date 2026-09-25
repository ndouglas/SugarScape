//! Artificial Anasazi (milestone 10): the Long House Valley, AD 800–1350,
//! after Janssen's ODD (2013) and "Understanding Artificial Anasazi" (JASSS
//! 12(4) 13, 2009), with the published replication's departures as named
//! switches. See docs/superpowers/specs/2026-09-25-anasazi-design.md and
//! its source extraction, docs/superpowers/specs/2026-09-25-anasazi-extraction.md
//! (cited below as §n, A-n, D-n). The model is written from those
//! descriptions; the NetLogo code was read only for data layouts.

pub mod config;
pub mod household;
pub mod random;
pub mod valley;
pub mod world;

pub use config::{presets, schema, AnasaziConfig, CornRange, Quirks};
pub use household::Household;
pub use world::{AnasaziInspection, AnasaziMode, AnasaziSnapshot, AnasaziWorld, SERIES};
