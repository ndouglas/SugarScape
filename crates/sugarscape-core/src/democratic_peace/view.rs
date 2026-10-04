use super::*;
use crate::{
    config::FieldError,
    model::{wrong_model, Model, ModelConfig, ModelKind},
    stats::Series,
};
use std::fmt::Write;
impl Model for DemocraticPeaceWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::DemocraticPeace(self.config.clone())
    }
    fn run(&mut self, t: u32) {
        DemocraticPeaceWorld::run(self, t)
    }
    fn tick(&self) -> u64 {
        self.tick
    }
    fn population(&self) -> usize {
        self.engine.states.len()
    }
    fn fingerprint(&self) -> u64 {
        self.economic_fingerprint()
    }
    fn size(&self) -> (u32, u32) {
        (self.config.width * 6, self.config.height * 6)
    }
    fn render(&self, mode: &str, _layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        if ![
            "",
            "territory",
            "governing_regime",
            "latent_regime",
            "resources",
            "alliances",
            "pariahs",
        ]
        .contains(&mode)
        {
            return Err(format!("unknown democratic_peace mode {mode}"));
        }
        let width = self.config.width as usize * 6;
        buf.clear();
        buf.resize(width * self.config.height as usize * 6 * 4, 255);
        for cell in &self.engine.cells {
            let state = &self.engine.states[&cell.owner];
            let k = (cell.owner.capital_cell as u32).wrapping_mul(2654435761);
            let mut color = [
                60 + (k % 160) as u8,
                60 + ((k >> 8) % 160) as u8,
                60 + ((k >> 16) % 160) as u8,
            ];
            match mode {
                "governing_regime" => {
                    color = if state.regime == Regime::Democratic {
                        [40, 155, 175]
                    } else {
                        [215, 100, 70]
                    }
                }
                "latent_regime" => {
                    color = if cell.latent_regime == Regime::Democratic {
                        [40, 155, 175]
                    } else {
                        [215, 100, 70]
                    }
                }
                "resources" => {
                    let v = (state.resources / 100.).clamp(0., 1.);
                    color = [30, (40. + 200. * v) as u8, 100];
                }
                "alliances" => {
                    color = self
                        .engine
                        .alliances
                        .iter()
                        .find(|a| a.members.contains(&cell.owner))
                        .map(|a| {
                            let v = a.threat_id.capital_cell as u32 * 17;
                            [70 + (v % 150) as u8, 70 + ((v / 7) % 150) as u8, 180]
                        })
                        .unwrap_or([100, 105, 115]);
                }
                "pariahs" => {
                    color = if self.engine.pariahs.contains_key(&cell.owner) {
                        [230, 60, 65]
                    } else {
                        [90, 120, 125]
                    }
                }
                _ => {}
            }
            let x = cell.id % self.config.width as usize;
            let y = cell.id / self.config.width as usize;
            for dy in 0..6 {
                for dx in 0..6 {
                    let i = ((y * 6 + dy) * width + x * 6 + dx) * 4;
                    let pixel = if dx == 0 || dy == 0 {
                        [20, 25, 30]
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
        series_names()
    }
    fn series(&self, n: &str) -> Option<Vec<f64>> {
        if n != "tick" && !SERIES.contains(&n) {
            return None;
        }
        Some(
            self.stats
                .history()
                .iter()
                .map(|s| s.value(n).unwrap_or(f64::NAN))
                .collect(),
        )
    }
    fn latest_value(&self, n: &str) -> Option<f64> {
        self.stats.latest().and_then(|s| s.value(n))
    }
    fn series_csv(&self) -> String {
        let mut out = format!("tick,{}\n", SERIES.join(","));
        for s in self.stats.history() {
            write!(out, "{}", s.tick).unwrap();
            for n in SERIES {
                out.push(',');
                if let Some(v) = s.value(n) {
                    write!(out, "{v}").unwrap();
                }
            }
            out.push('\n');
        }
        out
    }
    fn agents_csv(&self) -> String {
        let mut out = "cell,x,y,capital,generation,regime,latent_regime,resources\n".to_string();
        for c in &self.engine.cells {
            let s = &self.engine.states[&c.owner];
            let regime = if s.regime == Regime::Democratic {
                "democratic"
            } else {
                "predatory"
            };
            let latent = if c.latent_regime == Regime::Democratic {
                "democratic"
            } else {
                "predatory"
            };
            writeln!(
                out,
                "{},{},{},{},{},{},{},{}",
                c.id,
                c.id % self.config.width as usize,
                c.id / self.config.width as usize,
                c.owner.capital_cell,
                c.owner.sovereignty_generation,
                regime,
                latent,
                s.resources
            )
            .unwrap();
        }
        out
    }
    fn inspect_json(&self, x: u32, y: u32) -> Result<String, String> {
        if x >= self.config.width * 6 || y >= self.config.height * 6 {
            return Err("outside democratic_peace grid".into());
        }
        let id = ((y / 6) * self.config.width + x / 6) as usize;
        let cell = &self.engine.cells[id];
        let state = &self.engine.states[&cell.owner];
        let fronts: Vec<_> = self
            .engine
            .fronts
            .values()
            .filter(|f| f.states.contains(&cell.owner))
            .collect();
        let alliances: Vec<_> = self
            .engine
            .alliances
            .iter()
            .filter(|a| a.members.contains(&cell.owner) || a.threat_id == cell.owner)
            .collect();
        Ok(serde_json::json!({"model":"democratic_peace","config":self.config,"seed":self.seed,"cell":cell,"state":state,"members":state.members,"fronts":fronts,"alliances":alliances,"extraction":self.engine.extraction.iter().find(|r|r.state==cell.owner),"distance":self.engine.distance(&self.config,cell.owner,id)?,"pariah_sources":self.engine.pariahs.get(&cell.owner).cloned().unwrap_or_default(),"metrics":self.snapshot().metrics,"setup":self.setup,"period":self.engine.period,"periods":self.engine.period,"completed_periods":self.engine.period,"attempted_period":self.attempted,"last_tick_periods":self.last_tick_periods,"horizon_periods":self.config.horizon_periods,"finish_reason":self.result.as_ref().map(|o|&o.finish_reason),"invalidity":self.result.as_ref().and_then(|o|o.invalid_reason.as_ref()),"invalid_phase":self.result.as_ref().and_then(|o|o.invalid_phase.as_ref()),"outcome":self.result,"census":self.engine.counters,"last_structural_event":self.engine.last_structural_event,"events":self.engine.events,"events_dropped":self.engine.events_dropped,"path_priority":"lower_id_path_priority","agent":null}).to_string())
    }
    fn locate(&self, id: u64) -> Option<(u32, u32)> {
        (id < self.engine.cells.len() as u64).then_some((
            (id as u32 % self.config.width) * 6 + 3,
            (id as u32 / self.config.width) * 6 + 3,
        ))
    }
    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        let ModelConfig::DemocraticPeace(c) = next else {
            return Err(wrong_model(ModelKind::DemocraticPeace, &next));
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
