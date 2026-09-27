//! Arthur's "alphabet soup": the focal predictors of next week's attendance
//! from the past weeks' figures. Arthur names a few ("the same as last
//! week's", "a mirror image around 50 of last week's", "a (rounded) average
//! of the last four weeks", "the trend in last 8 weeks, bounded by 0, 100",
//! cycle detectors) and says he used "several dozen"; this is the stated
//! library of 48 (four dozen).

use serde::Serialize;

/// Weeks of history every predictor may read.
pub const LOOKBACK: usize = 12;

/// One focal predictor.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "weeks", rename_all = "snake_case")]
pub enum Predictor {
    /// The same as k weeks ago (k = 1: last week; k ≥ 2: a k-cycle detector).
    Same(u32),
    /// The mirror image around N/2 of k weeks ago.
    Mirror(u32),
    /// The rounded mean of the last k weeks.
    Mean(u32),
    /// The least-squares trend over the last k weeks, one week on, rounded.
    Trend(u32),
    /// The mirror image around N/2 of the rounded mean of the last k weeks.
    MirrorMean(u32),
}

/// Predictor families, for the Strategy color mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Family {
    Same,
    Mirror,
    Mean,
    Trend,
}

impl Predictor {
    pub fn family(self) -> Family {
        match self {
            Predictor::Same(_) => Family::Same,
            Predictor::Mirror(_) | Predictor::MirrorMean(_) => Family::Mirror,
            Predictor::Mean(_) => Family::Mean,
            Predictor::Trend(_) => Family::Trend,
        }
    }

    /// A plain description, for Inspect.
    pub fn describe(self) -> String {
        match self {
            Predictor::Same(1) => "the same as last week".into(),
            Predictor::Same(k) => format!("the same as {k} weeks ago"),
            Predictor::Mirror(1) => "last week mirrored".into(),
            Predictor::Mirror(k) => format!("{k} weeks ago, mirrored"),
            Predictor::Mean(k) => format!("the mean of the last {k} weeks"),
            Predictor::Trend(k) => format!("the trend of the last {k} weeks"),
            Predictor::MirrorMean(k) => format!("the mean of the last {k} weeks, mirrored"),
        }
    }

    /// Next week's attendance of `n` agents, from `history` (oldest first,
    /// at least `LOOKBACK` weeks), clamped to 0..=n.
    pub fn forecast(self, history: &[u32], n: u32) -> u32 {
        let t = history.len();
        let ago = |k: u32| f64::from(history[t - k as usize]);
        let mean = |k: u32| (1..=k).map(ago).sum::<f64>() / f64::from(k);
        let nf = f64::from(n);
        let x = match self {
            Predictor::Same(k) => ago(k),
            Predictor::Mirror(k) => nf - ago(k),
            Predictor::Mean(k) => mean(k).round(),
            Predictor::MirrorMean(k) => nf - mean(k).round(),
            Predictor::Trend(k) => {
                // Weeks 0..k − 1, oldest first; the fit's value at week k.
                let kf = f64::from(k);
                let xm = (kf - 1.0) / 2.0;
                let ym = mean(k);
                let (mut sxy, mut sxx) = (0.0, 0.0);
                for i in 0..k {
                    let dx = f64::from(i) - xm;
                    sxy += dx * (ago(k - i) - ym);
                    sxx += dx * dx;
                }
                (ym + sxy / sxx * (kf - xm)).round()
            }
        };
        x.clamp(0.0, nf) as u32
    }
}

/// The library, in a fixed order: 12 + 8 + 11 + 10 + 7 = 48 predictors.
pub fn library() -> Vec<Predictor> {
    let mut v = Vec::with_capacity(48);
    v.extend((1..=12).map(Predictor::Same));
    v.extend((1..=8).map(Predictor::Mirror));
    v.extend((2..=12).map(Predictor::Mean));
    v.extend((3..=12).map(Predictor::Trend));
    v.extend((2..=8).map(Predictor::MirrorMean));
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    const H: [u32; 12] = [10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 44, 35];

    #[test]
    fn the_library_has_four_dozen_distinct_predictors_reading_twelve_weeks() {
        let lib = library();
        assert_eq!(lib.len(), 48);
        for (i, p) in lib.iter().enumerate() {
            assert!(!lib[..i].contains(p), "{p:?} twice");
            p.forecast(&H, 100);
        }
        assert_eq!(LOOKBACK, 12);
    }

    #[test]
    fn arthur_s_examples_forecast_as_he_describes() {
        // His example history ends …, 56, 22, 35: last week 35, mirror 65.
        let h = [40, 56, 22, 35];
        let pad = |h: &[u32]| {
            let mut v = vec![0; LOOKBACK - h.len()];
            v.extend_from_slice(h);
            v
        };
        let h = pad(&h);
        assert_eq!(Predictor::Same(1).forecast(&h, 100), 35);
        assert_eq!(Predictor::Mirror(1).forecast(&h, 100), 65);
        assert_eq!(Predictor::Same(2).forecast(&h, 100), 22);
        // (40 + 56 + 22 + 35) / 4 = 38.25 → 38.
        assert_eq!(Predictor::Mean(4).forecast(&h, 100), 38);
        assert_eq!(Predictor::MirrorMean(4).forecast(&h, 100), 62);
    }

    #[test]
    fn trends_extrapolate_one_week_and_stay_in_bounds() {
        // A straight line 10, 20, …, 100 continues to 110 and is clamped.
        let up: Vec<u32> = (1..=12).map(|i| 10 * i.min(10)).collect();
        assert_eq!(Predictor::Trend(3).forecast(&up, 100), 100);
        let line = [0, 0, 0, 0, 0, 0, 0, 0, 0, 20, 30, 40];
        assert_eq!(Predictor::Trend(3).forecast(&line, 100), 50);
        let down = [0, 0, 0, 0, 0, 0, 0, 0, 0, 40, 20, 0];
        assert_eq!(
            Predictor::Trend(3).forecast(&down, 100),
            0,
            "−20 is clamped"
        );
        assert_eq!(Predictor::Trend(4).family(), Family::Trend);
    }

    #[test]
    fn predictors_describe_themselves() {
        assert_eq!(Predictor::Same(1).describe(), "the same as last week");
        assert_eq!(Predictor::Same(5).describe(), "the same as 5 weeks ago");
        assert_eq!(
            Predictor::Trend(8).describe(),
            "the trend of the last 8 weeks"
        );
    }
}
