//! The hoarding world. This is the skeleton: it holds the config and an empty
//! population and steps nothing. The season and the generations come next.

use crate::config::FieldError;
use crate::export;
use crate::model::{wrong_model, Model, ModelConfig, ModelKind};
use crate::rng::{self, SimRng};
use crate::stats::Stats;

use super::config::HoardConfig;
use super::stats::HoardSnapshot;

/// The frame's size in cells.
pub const WIDE: usize = 64;
pub const TALL: usize = 32;

const BACKGROUND: [u8; 4] = [16, 18, 24, 255];

#[derive(Clone)]
pub struct HoardWorld {
    pub config: HoardConfig,
    /// Completed generations' worth of ticks (none yet).
    pub tick: u64,
    #[allow(dead_code)] // the season and the generations draw from it
    rng: SimRng,
    pub stats: Stats<HoardSnapshot>,
}

impl HoardWorld {
    pub fn new(config: HoardConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        Ok(HoardWorld {
            config,
            tick: 0,
            rng: rng::seeded(seed),
            stats: Stats::default(),
        })
    }

    /// Steps nothing yet.
    pub fn run(&mut self, _ticks: u32) {}
}

impl Model for HoardWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Hoard(self.config.clone())
    }

    fn run(&mut self, ticks: u32) {
        HoardWorld::run(self, ticks);
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    fn population(&self) -> usize {
        0
    }

    /// FNV-1a over the tick.
    fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for b in self.tick.to_le_bytes() {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
        h
    }

    fn size(&self) -> (u32, u32) {
        (WIDE as u32, TALL as u32)
    }

    fn render(&self, mode: &str, _layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        if mode != "agents" {
            return Err(format!("unknown color mode {mode:?}"));
        }
        buf.clear();
        for _ in 0..WIDE * TALL {
            buf.extend_from_slice(&BACKGROUND);
        }
        Ok(())
    }

    fn latest_json(&self) -> String {
        serde_json::to_string(&self.stats.latest()).expect("snapshot serializes")
    }

    fn series_names(&self) -> Vec<String> {
        super::SERIES.iter().map(|s| s.to_string()).collect()
    }

    fn series(&self, name: &str) -> Option<Vec<f64>> {
        self.stats.series(name)
    }

    fn series_csv(&self) -> String {
        export::history_csv(&self.series_names(), self.stats.history())
    }

    fn agents_csv(&self) -> String {
        String::from("id,larder_probability,defense_propensity,stores\n")
    }

    fn inspect_json(&self, _x: u32, _y: u32) -> Result<String, String> {
        Err("there is nothing to inspect yet".into())
    }

    fn locate(&self, _id: u64) -> Option<(u32, u32)> {
        None
    }

    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        let ModelConfig::Hoard(next) = next else {
            return Err(wrong_model(ModelKind::Hoard, &next));
        };
        next.validate()?;
        let changes = self.config.structural_changes(&next);
        if !changes.is_empty() {
            return Err(changes);
        }
        self.config = next;
        Ok(())
    }
}
