use super::{Error, StrategicObservation};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub bits: u32,
}
impl Policy {
    pub fn new(bits: u32) -> Result<Self, Error> {
        if bits < 1 << 18 {
            Ok(Self { bits })
        } else {
            Err(Error::InvalidPolicy)
        }
    }
    pub fn calibration(&self, signal: bool) -> bool {
        ((self.bits >> u32::from(signal)) & 1) != 0
    }
    pub fn live(&self, c_signal: bool, c_report: bool, c_truth: bool, t_signal: bool) -> bool {
        let row = (u32::from(c_signal) << 3)
            | (u32::from(c_report) << 2)
            | (u32::from(c_truth) << 1)
            | u32::from(t_signal);
        ((self.bits >> (2 + row)) & 1) != 0
    }
    pub fn report(&self, view: &StrategicObservation) -> Result<bool, Error> {
        Self::new(self.bits)?;
        view.rules.validate()?;
        match (
            view.calibration_signal,
            view.calibration_reports,
            view.calibration_truth,
        ) {
            (None, None, None) => Ok(self.calibration(view.signal)),
            (Some(signal), Some(reports), Some(truth)) => {
                Ok(self.live(signal, reports[0], truth, view.signal))
            }
            _ => Err(Error::InvalidObservation(
                "live reporting requires calibration signal, reports and verified truth together",
            )),
        }
    }
    fn signal_control(calibration_invert: bool, live_invert: bool) -> Self {
        let mut bits = 0;
        for signal in [false, true] {
            if signal ^ calibration_invert {
                bits |= 1 << u32::from(signal);
            }
        }
        for row in 0..16 {
            if (row & 1 != 0) ^ live_invert {
                bits |= 1 << (2 + row);
            }
        }
        Self { bits }
    }
    pub fn copy() -> Self {
        Self::signal_control(false, false)
    }
    pub fn invert() -> Self {
        Self::signal_control(true, true)
    }
    pub fn positive() -> Self {
        Self {
            bits: (1 << 18) - 1,
        }
    }
    pub fn negative() -> Self {
        Self { bits: 0 }
    }
    pub fn calibration_copy_live_invert() -> Self {
        Self::signal_control(false, true)
    }
}
