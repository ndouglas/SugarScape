//! Rendering observes ownership, stocks and coalition membership without drawing randomness.
use super::{decision::Front, world::hash};
use super::{territory, world::PolarityWorld};
use crate::{
    config::FieldError,
    model::{wrong_model, Model, ModelConfig, ModelKind},
    stats::Series,
};
use std::fmt::Write;
pub fn render(w: &PolarityWorld, mode: &str, buf: &mut Vec<u8>) -> Result<(), String> {
    if !["", "territory", "resources", "strategy", "coalitions"].contains(&mode) {
        return Err(format!("unknown polarity mode {mode}"));
    }
    let width = w.config.width as usize * 6;
    buf.clear();
    buf.resize(width * w.config.height as usize * 6 * 4, 255);
    for c in &w.cells {
        let x = c.id % w.config.width as usize;
        let y = c.id / w.config.width as usize;
        let seed = (c.capital as u32).wrapping_mul(2654435761);
        let mut color = [
            70 + (seed % 160) as u8,
            70 + ((seed >> 8) % 160) as u8,
            70 + ((seed >> 16) % 160) as u8,
        ];
        if mode == "resources" {
            let t = (c.stock / 100.0).clamp(-1.0, 1.0);
            color = if t < 0.0 {
                [(100.0 - 140.0 * t) as u8, 30, 40]
            } else {
                [30, (40.0 + 190.0 * t) as u8, 100]
            };
        } else if mode == "strategy" {
            color = if c.predator {
                [230, 120, 40]
            } else {
                [60, 170, 100]
            };
        } else if mode == "coalitions" {
            let threat = w.coalition_threat(c.capital);
            color = threat
                .map(|id| {
                    let k = (id as u32).wrapping_mul(1664525);
                    [
                        80 + (k % 160) as u8,
                        80 + ((k >> 8) % 160) as u8,
                        80 + ((k >> 16) % 160) as u8,
                    ]
                })
                .unwrap_or([75, 80, 90]);
        }
        for dy in 0..6 {
            for dx in 0..6 {
                let i = ((y * 6 + dy) * width + x * 6 + dx) * 4;
                let border = (dx == 0 && (x == 0 || w.cells[c.id - 1].capital != c.capital))
                    || (dx == 5
                        && (x + 1 == w.config.width as usize
                            || w.cells[c.id + 1].capital != c.capital))
                    || (dy == 0
                        && (y == 0
                            || w.cells[c.id - w.config.width as usize].capital != c.capital))
                    || (dy == 5
                        && (y + 1 == w.config.height as usize
                            || w.cells[c.id + w.config.width as usize].capital != c.capital));
                let pixel = if border {
                    [20, 20, 25]
                } else if c.id == c.capital && (2..=3).contains(&dx) && (2..=3).contains(&dy) {
                    [250, 250, 250]
                } else if dx == 4 && dy == 4 {
                    if c.predator {
                        [255, 150, 30]
                    } else {
                        [60, 230, 100]
                    }
                } else {
                    color
                };
                buf[i..i + 3].copy_from_slice(&pixel);
            }
        }
    }
    let _ = territory::capitals(&w.cells);
    Ok(())
}

impl Model for PolarityWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Polarity(self.config.clone())
    }
    fn run(&mut self, ticks: u32) {
        PolarityWorld::run(self, ticks)
    }
    fn tick(&self) -> u64 {
        self.tick
    }
    fn population(&self) -> usize {
        territory::capitals(&self.cells).len()
    }
    fn fingerprint(&self) -> u64 {
        self.economic_fingerprint()
            ^ hash(
                &serde_json::to_vec(&(
                    self.tick,
                    &self.config,
                    &self.outcome,
                    &self.events,
                    self.events_dropped,
                ))
                .unwrap(),
            )
    }
    fn size(&self) -> (u32, u32) {
        (self.config.width * 6, self.config.height * 6)
    }
    fn render(&self, mode: &str, _layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        super::view::render(self, mode, buf)
    }
    fn latest_json(&self) -> String {
        serde_json::to_string(&self.stats.latest()).unwrap()
    }
    fn series_names(&self) -> Vec<String> {
        super::series_names()
    }
    fn series(&self, name: &str) -> Option<Vec<f64>> {
        self.stats.series(name)
    }
    fn latest_value(&self, name: &str) -> Option<f64> {
        self.stats.latest().and_then(|s| s.value(name))
    }
    fn series_csv(&self) -> String {
        crate::export::history_csv(&self.series_names(), self.stats.history())
    }
    fn agents_csv(&self) -> String {
        let mut s = "cell,x,y,capital,predator,stock\n".to_string();
        for c in &self.cells {
            writeln!(
                s,
                "{},{},{},{},{},{}",
                c.id,
                c.id % self.config.width as usize,
                c.id / self.config.width as usize,
                c.capital,
                c.predator,
                c.stock
            )
            .unwrap();
        }
        s
    }
    fn inspect_json(&self, x: u32, y: u32) -> Result<String, String> {
        if x >= self.config.width * 6 || y >= self.config.height * 6 {
            return Err("outside polarity grid".into());
        }
        let id = ((y / 6) * self.config.width + x / 6) as usize;
        let c = &self.cells[id];
        let capital = c.capital;
        let foreign: Vec<&Front> = self
            .fronts
            .values()
            .filter(|f| !f.key.domestic && (f.key.a == capital || f.key.b == capital))
            .collect();
        let domestic: Vec<&Front> = self
            .fronts
            .values()
            .filter(|f| f.key.domestic && f.key.a == capital)
            .collect();
        let coalition = self
            .coalitions
            .iter()
            .find(|(_, m)| m.contains(&capital))
            .map(|(threat, m)| serde_json::json!({"threat":threat,"members":m}));
        Ok(serde_json::json!({"model":"polarity","cell":c,"capital":capital,"province_stock":if c.id==capital{0.0}else{c.stock},"corporate_stock":self.cells[capital].stock,"members":territory::members(&self.cells,capital),"neighbors":territory::neighbors(&self.config,&self.cells,capital),"trust":self.trust.iter().filter(|((a,_),_)|*a==capital).map(|((_,b),v)|serde_json::json!({"neighbor":b,"trust":v})).collect::<Vec<_>>(),"threat":self.threats.get(&capital),"coalition":coalition,"foreign_fronts":foreign,"domestic_fronts":domestic,"last_event":self.last_event,"period":self.completed_periods(),"periods":self.completed_periods(),"attempted_period":self.period,"agent":null,"event_log_limit":self.config.event_log_limit,"horizon":self.config.horizon,"finish_reason":self.outcome.as_ref().map(|o|&o.finish_reason),"invalidity":self.outcome.as_ref().and_then(|o|o.invalid_reason.as_ref()),"event_log_enabled":self.config.event_log,"events":self.events,"events_dropped":self.events_dropped,"next_event_id":self.next_event}).to_string())
    }
    fn locate(&self, id: u64) -> Option<(u32, u32)> {
        if id >= self.cells.len() as u64 {
            None
        } else {
            Some((
                (id as u32 % self.config.width) * 6 + 3,
                (id as u32 / self.config.width) * 6 + 3,
            ))
        }
    }
    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        let ModelConfig::Polarity(c) = next else {
            return Err(wrong_model(ModelKind::Polarity, &next));
        };
        c.validate()?;
        let a = serde_json::to_value(&self.config).unwrap();
        let b = serde_json::to_value(&c).unwrap();
        let errors: Vec<FieldError> = a
            .as_object()
            .unwrap()
            .iter()
            .filter(|(k, v)| b[*k] != **v)
            .map(|(k, _)| FieldError::new(k, "changes only on reset"))
            .collect();
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    fn finished(&self) -> bool {
        self.outcome.is_some()
    }
    fn holds_when_finished(&self) -> bool {
        self.outcome.as_ref().is_some_and(|o| o.valid)
    }
}
