//! A claim about a preset or sweep, and the judges that decide it.

use serde::Serialize;

use crate::stats;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Source {
    Book,
    App,
    Comment,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Verdict {
    Holds,
    Weak,
    Fails,
    Untestable,
    Error,
}

#[derive(Clone, Debug, Serialize)]
pub struct Outcome {
    pub verdict: Verdict,
    pub measured: String,
    pub detail: String,
}

impl Outcome {
    pub fn with(mut self, detail: &str) -> Self {
        if !self.detail.is_empty() {
            self.detail.push(' ');
        }
        self.detail.push_str(detail);
        self
    }
}

pub struct Claim {
    /// `<item>.<short-name>`, e.g. `ii-2.capacity`.
    pub id: &'static str,
    /// A preset or sweep id.
    pub item: &'static str,
    pub source: Source,
    /// Where the claim is stated (a spec path, `presets.rs` description, or
    /// "book, from memory: …").
    pub citation: &'static str,
    /// The claim in the source's words.
    pub text: &'static str,
    pub check: fn(&[u64]) -> Outcome,
}

const MIN_SEEDS: usize = 5;

fn summary(v: &[f64]) -> String {
    format!(
        "median {:.4} (IQR {:.4}–{:.4})",
        stats::median(v),
        stats::quantile(v, 0.25),
        stats::quantile(v, 0.75)
    )
}

pub fn untestable(reason: &str) -> Outcome {
    Outcome {
        verdict: Verdict::Untestable,
        measured: String::new(),
        detail: reason.into(),
    }
}

fn too_few(n: usize) -> Outcome {
    untestable(&format!(
        "only {n} seeds gave a finite value (need {MIN_SEEDS})"
    ))
}

/// Holds when ≥ 80% of seeds are in [lo, hi] (widened by 10% when `about`).
pub fn range(values: &[f64], lo: f64, hi: f64, about: bool) -> Outcome {
    let v = stats::finite(values);
    if v.len() < MIN_SEEDS {
        return too_few(v.len());
    }
    let (lo, hi) = if about {
        (lo - 0.1 * lo.abs(), hi + 0.1 * hi.abs())
    } else {
        (lo, hi)
    };
    let frac = stats::frac_in(&v, lo, hi);
    let inside = (frac * v.len() as f64).round() as usize;
    let verdict = if frac >= 0.8 {
        Verdict::Holds
    } else if frac >= 0.5 {
        Verdict::Weak
    } else {
        Verdict::Fails
    };
    Outcome {
        verdict,
        measured: format!(
            "{}; {inside}/{} in [{lo:.4}, {hi:.4}]",
            summary(&v),
            v.len()
        ),
        detail: String::new(),
    }
}

/// Claim: `a` exceeds `b`. One-sided Mann–Whitney at p < 0.01.
pub fn greater(a: &[f64], b: &[f64], a_name: &str, b_name: &str) -> Outcome {
    let (a, b) = (stats::finite(a), stats::finite(b));
    if a.len().min(b.len()) < MIN_SEEDS {
        return too_few(a.len().min(b.len()));
    }
    let p = stats::mw_greater(&a, &b);
    let verdict = if p < 0.01 {
        Verdict::Holds
    } else if stats::median(&a) > stats::median(&b) {
        Verdict::Weak
    } else {
        Verdict::Fails
    };
    Outcome {
        verdict,
        measured: format!(
            "{a_name} {}; {b_name} {}; one-sided Mann–Whitney p = {p:.2e}; n = {} vs {}",
            summary(&a),
            summary(&b),
            a.len(),
            b.len()
        ),
        detail: String::new(),
    }
}

/// Claim: `a` and `b` are about the same. TOST at 0.05 against `margin`
/// (default 10% of the pooled mean).
pub fn equivalent(
    a: &[f64],
    b: &[f64],
    margin: Option<f64>,
    a_name: &str,
    b_name: &str,
) -> Outcome {
    let (a, b) = (stats::finite(a), stats::finite(b));
    if a.len().min(b.len()) < MIN_SEEDS {
        return too_few(a.len().min(b.len()));
    }
    let margin = margin
        .unwrap_or_else(|| 0.1 * stats::mean(&[a.as_slice(), b.as_slice()].concat()).abs());
    let p_eq = stats::tost(&a, &b, margin);
    let p_diff = stats::mw_two_sided(&a, &b);
    let verdict = if p_eq < 0.05 {
        Verdict::Holds
    } else if p_diff < 0.01 {
        Verdict::Fails
    } else {
        Verdict::Weak
    };
    Outcome {
        verdict,
        measured: format!(
            "{a_name} {}; {b_name} {}; TOST p = {p_eq:.2e} (margin {margin:.4}); two-sided Mann–Whitney p = {p_diff:.2e}; n = {} vs {}",
            summary(&a),
            summary(&b),
            a.len(),
            b.len()
        ),
        detail: String::new(),
    }
}

/// Several judged parts of one statement ("at every vision"): the worst
/// verdict wins (Fails, then Weak, then Untestable, then Holds; Error first).
pub fn all_of(parts: Vec<(String, Outcome)>) -> Outcome {
    let rank = |v: Verdict| match v {
        Verdict::Error => 4,
        Verdict::Fails => 3,
        Verdict::Weak => 2,
        Verdict::Untestable => 1,
        Verdict::Holds => 0,
    };
    let verdict = parts
        .iter()
        .map(|(_, o)| o.verdict)
        .max_by_key(|v| rank(*v))
        .unwrap_or(Verdict::Untestable);
    let measured = parts
        .iter()
        .map(|(label, o)| format!("[{label}: {:?}] {}", o.verdict, o.measured))
        .collect::<Vec<_>>()
        .join(" ");
    let detail = parts
        .iter()
        .filter(|(_, o)| !o.detail.is_empty())
        .map(|(label, o)| format!("[{label}] {}", o.detail))
        .collect::<Vec<_>>()
        .join(" ");
    Outcome { verdict, measured, detail }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_counts_seeds_and_widens_about() {
        let v: Vec<f64> = (0..20).map(f64::from).collect(); // 0..19
        assert_eq!(range(&v, 0.0, 15.0, false).verdict, Verdict::Holds); // 16/20
        assert_eq!(range(&v, 0.0, 11.0, false).verdict, Verdict::Weak); // 12/20
        assert_eq!(range(&v, 0.0, 5.0, false).verdict, Verdict::Fails); // 6/20
        // about: [10, 15] becomes [9, 16.5], 8/20.
        assert!(range(&v, 10.0, 15.0, true).measured.contains("8/20"));
    }

    #[test]
    fn judges_drop_nan_and_need_five_seeds() {
        let mut v = vec![f64::NAN; 16];
        v.extend([1.0, 1.0, 1.0, 1.0]);
        assert_eq!(range(&v, 0.0, 2.0, false).verdict, Verdict::Untestable);
        let a = [5.0, 6.0, 7.0, 8.0, 9.0, f64::NAN];
        let b = [1.0, 2.0, 3.0, 4.0, 0.0, 0.5];
        let o = greater(&a, &b, "a", "b");
        assert!(o.measured.contains("n = 5 vs 6"), "{}", o.measured);
    }

    #[test]
    fn greater_needs_significance_and_direction() {
        let hi: Vec<f64> = (0..20).map(|i| f64::from(i) + 30.0).collect();
        let lo: Vec<f64> = (0..20).map(f64::from).collect();
        assert_eq!(greater(&hi, &lo, "hi", "lo").verdict, Verdict::Holds);
        assert_eq!(greater(&lo, &hi, "lo", "hi").verdict, Verdict::Fails);
        let near: Vec<f64> = lo.iter().map(|x| x + 1.0).collect();
        assert_eq!(greater(&near, &lo, "near", "lo").verdict, Verdict::Weak);
    }

    #[test]
    fn equivalent_distinguishes_all_three() {
        let a: Vec<f64> = (0..20).map(|i| 100.0 + f64::from(i % 5)).collect();
        let close: Vec<f64> = a.iter().map(|x| x + 0.5).collect();
        let far: Vec<f64> = a.iter().map(|x| x + 40.0).collect();
        assert_eq!(equivalent(&a, &close, None, "a", "b").verdict, Verdict::Holds);
        assert_eq!(equivalent(&a, &far, None, "a", "b").verdict, Verdict::Fails);
        let noisy: Vec<f64> = (0..20).map(|i| 100.0 + f64::from(i * 7 % 40)).collect();
        let noisy2: Vec<f64> = noisy.iter().map(|x| x + 8.0).collect();
        assert_eq!(equivalent(&noisy, &noisy2, None, "a", "b").verdict, Verdict::Weak);
    }
}
