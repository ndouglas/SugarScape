//! Read-only host adapter: rendering and inspection do not consume random draws.
use super::*;
use crate::{
    config::FieldError,
    model::{wrong_model, Model, ModelConfig, ModelKind},
    stats::Series,
};
use std::fmt::Write;
impl Model for GeosimWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Geosim(self.config.clone())
    }
    fn run(&mut self, t: u32) {
        GeosimWorld::run(self, t)
    }
    fn tick(&self) -> u64 {
        self.tick
    }
    fn population(&self) -> usize {
        self.states.len()
    }
    fn fingerprint(&self) -> u64 {
        self.economic_fingerprint()
            ^ super::world::hash(
                &serde_json::to_vec(&(self.tick, &self.config, &self.events, self.events_dropped))
                    .unwrap(),
            )
    }
    fn size(&self) -> (u32, u32) {
        (self.config.width * 6, self.config.height * 6)
    }
    fn render(&self, mode: &str, _layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        if !["", "territory", "capacity", "technology", "alert", "wars"].contains(&mode) {
            return Err(format!("unknown geosim mode {mode}"));
        }
        let width = self.config.width as usize * 6;
        buf.clear();
        buf.resize(width * self.config.height as usize * 6 * 4, 255);
        for cell in &self.cells {
            let x = cell.id % self.config.width as usize;
            let y = cell.id / self.config.width as usize;
            let state = &self.states[&cell.owner];
            let k = (cell.owner.capital_cell as u32).wrapping_mul(2654435761);
            let mut color = [
                70 + (k % 160) as u8,
                70 + ((k >> 8) % 160) as u8,
                70 + ((k >> 16) % 160) as u8,
            ];
            if mode == "capacity" {
                let t = (state.capacity.unwrap_or(0.0) / 100.0).clamp(0.0, 1.0);
                color = [30, (40.0 + 190.0 * t) as u8, 100];
            } else if mode == "technology" {
                let t = (state.threshold
                    / (self.config.distance_threshold + self.config.shock_shift).max(1.0))
                .clamp(0.0, 1.0);
                color = [(220.0 * t) as u8, 80, (220.0 * (1.0 - t)) as u8];
            } else if mode == "alert" {
                color = if state.alert {
                    [230, 100, 40]
                } else {
                    [70, 130, 100]
                };
            } else if mode == "wars" {
                color = if self
                    .tracker
                    .active
                    .values()
                    .any(|w| w.participants.iter().any(|p| p.state == cell.owner))
                {
                    [230, 80, 70]
                } else {
                    [80, 100, 120]
                };
            }
            for dy in 0..6 {
                for dx in 0..6 {
                    let i = ((y * 6 + dy) * width + x * 6 + dx) * 4;
                    let border = dx == 0 || dy == 0;
                    let pixel = if border {
                        [20, 20, 25]
                    } else if cell.id == cell.owner.capital_cell
                        && (2..=3).contains(&dx)
                        && (2..=3).contains(&dy)
                    {
                        [250, 250, 250]
                    } else {
                        color
                    };
                    buf[i..i + 3].copy_from_slice(&pixel);
                }
            }
        }
        Ok(())
    }
    fn latest_json(&self) -> String {
        serde_json::to_string(&self.stats.latest()).unwrap()
    }
    fn series_names(&self) -> Vec<String> {
        super::series_names()
    }
    fn series(&self, n: &str) -> Option<Vec<f64>> {
        self.stats.series(n)
    }
    fn latest_value(&self, n: &str) -> Option<f64> {
        self.stats.latest().and_then(|s| s.value(n))
    }
    fn series_csv(&self) -> String {
        crate::export::history_csv(&self.series_names(), self.stats.history())
    }
    fn agents_csv(&self) -> String {
        let mut out = "cell,x,y,capital,generation,capacity,threshold,alert\n".to_string();
        for cell in &self.cells {
            let s = &self.states[&cell.owner];
            writeln!(
                out,
                "{},{},{},{},{},{},{},{}",
                cell.id,
                cell.id % self.config.width as usize,
                cell.id / self.config.width as usize,
                cell.owner.capital_cell,
                cell.owner.sovereignty_generation,
                s.capacity.map(|v| v.to_string()).unwrap_or_default(),
                s.threshold,
                s.alert
            )
            .unwrap();
        }
        out
    }
    fn inspect_json(&self, x: u32, y: u32) -> Result<String, String> {
        if x >= self.config.width * 6 || y >= self.config.height * 6 {
            return Err("outside geosim grid".into());
        }
        let id = ((y / 6) * self.config.width + x / 6) as usize;
        let c = &self.cells[id];
        let s = &self.states[&c.owner];
        let fronts: Vec<_> = self
            .fronts
            .values()
            .filter(|f| f.states.contains(&c.owner))
            .collect();
        let wars: Vec<_> = self
            .tracker
            .active
            .values()
            .filter(|w| w.participants.iter().any(|p| p.state == c.owner))
            .collect();
        Ok(serde_json::json!({"model":"geosim","cell":c,"state":s,"members":self.members[&c.owner],"distance":super::territory::distance(&self.config,c.owner.capital_cell,id),"projection":self.projection(c.owner,id),"resource_recurrence":self.resource_updates.iter().find(|r|r.state==c.owner),"fronts":fronts,"wars":wars,"period":self.completed,"periods":self.completed,"attempted_period":self.period,"counting_start":self.counting_start(),"finish_reason":self.result.as_ref().map(|o|&o.finish_reason),"invalidity":self.result.as_ref().and_then(|o|o.invalid_reason.as_ref()),"outcome":self.result,"last_structural_event":self.last_structural_event,"events":self.events,"events_dropped":self.events_dropped,"agent":null}).to_string())
    }
    fn locate(&self, id: u64) -> Option<(u32, u32)> {
        (id < self.cells.len() as u64).then(|| {
            (
                (id as u32 % self.config.width) * 6 + 3,
                (id as u32 / self.config.width) * 6 + 3,
            )
        })
    }
    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        let ModelConfig::Geosim(c) = next else {
            return Err(wrong_model(ModelKind::Geosim, &next));
        };
        c.validate()?;
        if c == self.config {
            Ok(())
        } else {
            Err(vec![FieldError::new("config", "changes only on reset")])
        }
    }
    fn finished(&self) -> bool {
        self.result.is_some()
    }
    fn holds_when_finished(&self) -> bool {
        self.result.as_ref().is_some_and(|o| o.valid)
    }
}
