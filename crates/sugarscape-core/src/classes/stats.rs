//! The Emergence of Classes model's statistics, demands, best replies and
//! regimes (our reading of Axtell, Epstein & Young's pictures).

use serde::Serialize;

use crate::stats::Series;

/// The three demands, as indices: L, M, H.
pub const L: u8 = 0;
pub const M: u8 = 1;
pub const H: u8 = 2;

/// The statistics series, in the order the CSV and the page list them.
pub const SERIES: [&str; 14] = [
    "mean_payoff",
    "m_share",
    "outcome_mm",
    "outcome_hl",
    "outcome_fail",
    "outcome_waste",
    "regime",
    "segregated",
    "equity_at",
    "first_attractor",
    "payoff_dark",
    "payoff_light",
    "payoff_inter",
    "realized_noise",
];

/// Regime codes (the `regime` series).
pub const MIXED: u8 = 0;
pub const EQUITY: u8 = 1;
pub const FRACTIOUS: u8 = 2;
pub const CLASSES: u8 = 3;
pub const EQUITY_BETWEEN: u8 = 4;
pub const DIVIDED_BELOW: u8 = 5;

/// One period's statistics.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct ClassesSnapshot {
    pub tick: u64,
    /// Mean payoff per agent per match this period.
    pub mean_payoff: f64,
    /// M's among every remembered demand.
    pub m_share: f64,
    /// Shares of this period's matches: both M; one H and one L; demands
    /// over 100; demands under 100.
    pub outcome_mm: f64,
    pub outcome_hl: f64,
    pub outcome_fail: f64,
    pub outcome_waste: f64,
    pub regime: u8,
    /// 1 in classes or with equity above and division below, else 0.
    pub segregated: u8,
    /// The first period in equity, else this period.
    pub equity_at: u64,
    /// The first of equity (1) or fractiousness (2) reached, else 0.
    pub first_attractor: u8,
    /// With tags: each tag's mean payoff per match, and the darks' mean
    /// payoff against lights minus the lights' against darks (0 without).
    pub payoff_dark: f64,
    pub payoff_light: f64,
    pub payoff_inter: f64,
    /// The share of demands that differed from the best reply (AEY's note
    /// 9: 2ε/3 once replies are unique).
    pub realized_noise: f64,
}

impl Series for ClassesSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }

    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "mean_payoff" => self.mean_payoff,
            "m_share" => self.m_share,
            "outcome_mm" => self.outcome_mm,
            "outcome_hl" => self.outcome_hl,
            "outcome_fail" => self.outcome_fail,
            "outcome_waste" => self.outcome_waste,
            "regime" => f64::from(self.regime),
            "segregated" => f64::from(self.segregated),
            "equity_at" => self.equity_at as f64,
            "first_attractor" => f64::from(self.first_attractor),
            "payoff_dark" => self.payoff_dark,
            "payoff_light" => self.payoff_light,
            "payoff_inter" => self.payoff_inter,
            "realized_noise" => self.realized_noise,
            _ => return None,
        })
    }
}

/// The demands whose expected payoff against remembered counts `c` (L, M,
/// H) is highest (AEY), as a bit set of demand indices; all three when `c`
/// is empty. `low` is the L demand; H = 100 − L.
pub fn expected_best(c: [u32; 3], low: u32) -> u8 {
    let total = c[0] + c[1] + c[2];
    if total == 0 {
        return 0b111;
    }
    // Expected payoffs × total, exactly: L always gets L; M gets 50 unless
    // the opponent demands H; H gets H only against L.
    let e = [low * total, 50 * (c[0] + c[1]), (100 - low) * c[0]];
    let best = *e.iter().max().unwrap();
    (0..3).filter(|&d| e[d] == best).fold(0, |s, d| s | 1 << d)
}

/// The best replies to the most frequent remembered demands (Poza et al.'s
/// mode rule: L → H, M → M, H → L), as a bit set; all three when empty.
pub fn mode_best(c: [u32; 3]) -> u8 {
    let top = *c.iter().max().unwrap();
    if top == 0 {
        return 0b111;
    }
    (0..3)
        .filter(|&d| c[d] == top)
        .fold(0, |s, d| s | 1 << (2 - d))
}

/// A memory's view for classification: its counts (L, M, H).
pub type View = [u32; 3];

/// Whether every nonempty view in `views` holds at least (1 − ε) of its
/// length in M's, and at least one is nonempty: AEY's transition target
/// ("all agents have at least (1 − ε)m instances of M in their memories"),
/// behind `equity_at`.
pub fn equity(views: &[View], noise: f64) -> bool {
    let mut any = false;
    for v in views {
        let total = v[0] + v[1] + v[2];
        if total == 0 {
            continue;
        }
        any = true;
        if f64::from(v[1]) < (1.0 - noise) * f64::from(total) {
            return false;
        }
    }
    any
}

/// Whether every nonempty view's best replies (bit sets from `best`) lie
/// within `allowed`, and at least one is nonempty.
pub fn all_within(views: &[View], best: impl Fn(View) -> u8, allowed: u8) -> bool {
    let mut any = false;
    for &v in views {
        if v == [0, 0, 0] {
            continue;
        }
        any = true;
        if best(v) & !allowed != 0 {
            return false;
        }
    }
    any
}

/// The regime of the tagged contexts (intra-dark, intra-light, the darks'
/// views of lights, the lights' of darks), or of one type (`intra_dark`
/// alone, the rest empty), read from the best-reply regions agents sit in,
/// as AEY's pictures are: equity where every agent's best reply is M.
pub fn regime(tags: bool, views: [&[View]; 4], best: impl Fn(View) -> u8 + Copy) -> u8 {
    let [dd, ll, dl, ld] = views;
    let not_m = 1 << L | 1 << H;
    let fractious = |v: &[View]| all_within(v, best, not_m);
    let equity = |v: &[View]| all_within(v, best, 1 << M);
    if !tags {
        return if equity(dd) {
            EQUITY
        } else if fractious(dd) {
            FRACTIOUS
        } else {
            MIXED
        };
    }
    let inter: Vec<View> = dl.iter().chain(ld).copied().collect();
    let (eq_d, eq_l, eq_x) = (equity(dd), equity(ll), equity(&inter));
    let aggressive = |v: &[View]| all_within(v, best, 1 << H);
    let submissive = |v: &[View]| all_within(v, best, 1 << L);
    let split = (aggressive(dl) && submissive(ld)) || (submissive(dl) && aggressive(ld));
    if eq_d && eq_l && eq_x {
        EQUITY
    } else if eq_d && eq_l && split {
        CLASSES
    } else if split && ((eq_d && fractious(ll)) || (eq_l && fractious(dd))) {
        DIVIDED_BELOW
    } else if eq_x {
        EQUITY_BETWEEN
    } else if fractious(dd) && fractious(ll) && fractious(&inter) {
        FRACTIOUS
    } else {
        MIXED
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expected_payoffs_pick_the_papers_best_replies() {
        // All H remembered: only L gets anything.
        assert_eq!(expected_best([0, 0, 10], 30), 1 << L);
        // All L: H.
        assert_eq!(expected_best([10, 0, 0], 30), 1 << H);
        // All M: M.
        assert_eq!(expected_best([0, 10, 0], 30), 1 << M);
        // L and M tie where 30·total = 50·(L's + M's): 3 of 5 remembered
        // demands L or M.
        assert_eq!(expected_best([0, 3, 2], 30), 1 << L | 1 << M);
        // Empty: anything.
        assert_eq!(expected_best([0, 0, 0], 30), 0b111);
    }

    #[test]
    fn m_is_never_a_best_reply_without_m_in_memory() {
        for low in [5, 30, 45] {
            for l in 0..=20 {
                let b = expected_best([l, 0, 20 - l], low);
                assert_eq!(b & 1 << M, 0, "low {low}, {l} L's");
            }
        }
    }

    #[test]
    fn the_mode_rule_answers_the_commonest_demand() {
        assert_eq!(mode_best([5, 3, 2]), 1 << H);
        assert_eq!(mode_best([2, 5, 3]), 1 << M);
        assert_eq!(mode_best([2, 3, 5]), 1 << L);
        assert_eq!(mode_best([4, 4, 2]), 1 << H | 1 << M);
        assert_eq!(mode_best([0, 0, 0]), 0b111);
    }

    #[test]
    fn regimes_follow_the_papers_pictures() {
        let best = |v| expected_best(v, 30);
        let m: View = [0, 10, 0];
        let h: View = [0, 0, 10];
        let l: View = [10, 0, 0];
        // Mostly L, some M: H is the best reply (an aggressive agent among compromisers).
        let mixed: View = [6, 2, 2];
        assert_eq!(
            regime(false, [&[m, [0, 9, 1]], &[], &[], &[]], best),
            EQUITY
        );
        assert_eq!(regime(false, [&[h, l], &[], &[], &[]], best), FRACTIOUS);
        assert_eq!(regime(false, [&[m, mixed], &[], &[], &[]], best), MIXED);
        // Tags: darks remember lights as L (they demand H), lights remember darks as H.
        assert_eq!(regime(true, [&[m], &[m], &[m], &[m]], best), EQUITY);
        assert_eq!(regime(true, [&[m], &[m], &[l], &[h]], best), CLASSES);
        assert_eq!(
            regime(true, [&[m], &[h, l], &[l], &[h]], best),
            DIVIDED_BELOW
        );
        assert_eq!(
            regime(true, [&[m], &[h, l], &[m], &[m]], best),
            EQUITY_BETWEEN
        );
        assert_eq!(regime(true, [&[h], &[l], &[h, l], &[l]], best), FRACTIOUS);
        assert_eq!(regime(true, [&[mixed], &[m], &[m], &[l]], best), MIXED);
    }
}
