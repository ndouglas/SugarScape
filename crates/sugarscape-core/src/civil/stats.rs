//! The civil violence model's statistics and the paper's outburst
//! bookkeeping (Figs. 5 and 7).

use serde::Serialize;

use crate::stats::Series;

/// The statistics series, in the order the CSV and the page list them.
pub const SERIES: [&str; 15] = [
    "population",
    "active",
    "quiet",
    "jailed",
    "cops",
    "legitimacy",
    "mean_grievance",
    "tension",
    "outbursts",
    "mean_wait",
    "mean_activation",
    "blue",
    "green",
    "killed",
    "extinction",
];

/// One tick's statistics.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct CivilSnapshot {
    pub tick: u64,
    /// Agents alive, free or jailed.
    pub population: u32,
    /// Free agents that are active.
    pub active: u32,
    /// Free agents that are quiet.
    pub quiet: u32,
    pub jailed: u32,
    pub cops: u32,
    pub legitimacy: f64,
    /// Mean G over free agents (0 with none).
    pub mean_grievance: f64,
    /// The paper's ripeness index Ḡ·B̄/R̄ over free agents, B̄ the quiet
    /// share (0 with none).
    pub tension: f64,
    /// Outbursts that have ended.
    pub outbursts: u32,
    /// Mean ticks from one outburst's end to the next one's start (NaN
    /// before the first such wait; JSON null).
    pub mean_wait: f64,
    /// Mean total activation (the sum of actives over its ticks) of the
    /// outbursts that have ended (NaN before the first; JSON null).
    pub mean_activation: f64,
    /// Agents of each group, free or jailed (0 in Model I).
    pub blue: u32,
    pub green: u32,
    /// Agents killed this tick (Model II).
    pub killed: u32,
    /// The first tick at which a group was gone (Model II), else this tick.
    pub extinction: u64,
}

impl Series for CivilSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }

    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "population" => f64::from(self.population),
            "active" => f64::from(self.active),
            "quiet" => f64::from(self.quiet),
            "jailed" => f64::from(self.jailed),
            "cops" => f64::from(self.cops),
            "legitimacy" => self.legitimacy,
            "mean_grievance" => self.mean_grievance,
            "tension" => self.tension,
            "outbursts" => f64::from(self.outbursts),
            "mean_wait" => self.mean_wait,
            "mean_activation" => self.mean_activation,
            "blue" => f64::from(self.blue),
            "green" => f64::from(self.green),
            "killed" => f64::from(self.killed),
            "extinction" => self.extinction as f64,
            _ => return None,
        })
    }
}

/// Outbursts as the paper counts them: one starts on the first tick with
/// more than `threshold` actives and ends on the first later tick with
/// fewer; its total activation sums the actives of every tick from its
/// start to the tick before its end. A wait runs from one outburst's end to
/// the next one's start.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Outbursts {
    /// The running outburst's activation so far, if one is under way.
    current: Option<u64>,
    last_end: Option<u64>,
    pub ended: u32,
    activation_sum: u64,
    waits: u32,
    wait_sum: u64,
}

impl Outbursts {
    /// Records tick `tick`'s actives.
    pub fn record(&mut self, tick: u64, active: u32, threshold: u32) {
        match self.current {
            None if active > threshold => {
                if let Some(end) = self.last_end {
                    self.waits += 1;
                    self.wait_sum += tick - end;
                }
                self.current = Some(u64::from(active));
            }
            Some(sum) if active < threshold => {
                self.ended += 1;
                self.activation_sum += sum;
                self.last_end = Some(tick);
                self.current = None;
            }
            Some(sum) => self.current = Some(sum + u64::from(active)),
            None => {}
        }
    }

    pub fn mean_wait(&self) -> f64 {
        if self.waits == 0 {
            f64::NAN
        } else {
            self.wait_sum as f64 / f64::from(self.waits)
        }
    }

    pub fn mean_activation(&self) -> f64 {
        if self.ended == 0 {
            f64::NAN
        } else {
            self.activation_sum as f64 / f64::from(self.ended)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outbursts_follow_the_papers_example() {
        // The paper's example flare-up of 60, 100, 120, 95, 80 (which it
        // says sums to 500 and averages 100; the sum is 455), then a wait of
        // 3 ticks and a second outburst.
        let mut o = Outbursts::default();
        let actives = [10, 60, 100, 120, 95, 80, 20, 30, 40, 55, 70, 50, 49];
        for (t, &a) in actives.iter().enumerate() {
            o.record(t as u64, a, 50);
        }
        assert_eq!(o.ended, 2);
        // The first ends at t = 6, the second starts at t = 9. At exactly
        // 50 the second is still under way (not "below 50"): 55 + 70 + 50.
        assert_eq!(o.mean_wait(), 3.0);
        assert_eq!(o.mean_activation(), (455.0 + 175.0) / 2.0);
    }

    #[test]
    fn nothing_is_measured_before_the_first_outburst() {
        let mut o = Outbursts::default();
        o.record(0, 50, 50);
        o.record(1, 51, 50);
        assert_eq!(o.ended, 0);
        assert!(o.mean_wait().is_nan() && o.mean_activation().is_nan());
        o.record(2, 3, 50);
        assert_eq!(o.mean_activation(), 51.0);
        assert!(o.mean_wait().is_nan(), "no second outburst yet");
    }
}
