//! The strategy classes and their decisions: NS98's k (recipient's score at
//! least k), AND and OR (with the donor's own score below h); LH01's h (own
//! score below h), binary scorers, Sugden's standing and the q strategies.

use serde::{Deserialize, Serialize};

/// The lowest and highest k and h (NS98: k −5 … +6; FAIR23 draws h alike).
pub const K_MIN: i8 = -5;
pub const K_MAX: i8 = 6;

/// A class of strategies a run allows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Class {
    /// Help when the recipient's score is at least k (NS98).
    K,
    /// Help when one's own score is below h (LH01).
    H,
    /// Help when the recipient's score is at least k and one's own is below h.
    And,
    /// Help when the recipient's score is at least k or one's own is below h.
    Or,
    /// NS98's strategies that "only consider their own image": the same rule
    /// as `h`, counted apart.
    OwnOnly,
    /// LH01 Fig. 4's scorers with scores 0 and −1: cooperators (k −1),
    /// discriminators (k 0) and defectors (k 1).
    Binary,
    /// Sugden's standing strategy (LH01 §3).
    Standing,
    /// LH01's q strategies: help when helping is estimated to raise the
    /// chance of being helped by more than Δq.
    Q,
}

impl Class {
    pub const ALL: [Class; 8] = [
        Class::K,
        Class::H,
        Class::And,
        Class::Or,
        Class::OwnOnly,
        Class::Binary,
        Class::Standing,
        Class::Q,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Class::K => "k",
            Class::H => "h",
            Class::And => "and",
            Class::Or => "or",
            Class::OwnOnly => "own_only",
            Class::Binary => "binary",
            Class::Standing => "standing",
            Class::Q => "q",
        }
    }

    /// Every strategy of the class, in a fixed order (AND and OR: k outer, h inner).
    pub fn members(self) -> Vec<Strategy> {
        let ks = || K_MIN..=K_MAX;
        match self {
            Class::K => ks().map(Strategy::K).collect(),
            Class::H => ks().map(Strategy::H).collect(),
            Class::OwnOnly => ks().map(Strategy::OwnOnly).collect(),
            Class::And => ks()
                .flat_map(|k| ks().map(move |h| Strategy::And { k, h }))
                .collect(),
            Class::Or => ks()
                .flat_map(|k| ks().map(move |h| Strategy::Or { k, h }))
                .collect(),
            Class::Binary => (-1..=1).map(Strategy::Binary).collect(),
            Class::Standing => vec![Strategy::Standing],
            Class::Q => (1..=99).map(Strategy::Q).collect(),
        }
    }
}

/// One genotype. On the wire: `{"k": 0}`, `{"h": 1}`, `{"and": {"k": 0,
/// "h": 1}}`, `{"or": {…}}`, `{"own_only": 1}`, `{"binary": 0}`,
/// `"standing"`, `{"q": 25}` (Δq in hundredths).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Strategy {
    K(i8),
    H(i8),
    And { k: i8, h: i8 },
    Or { k: i8, h: i8 },
    OwnOnly(i8),
    Binary(i8),
    Standing,
    Q(u8),
}

/// What a donor knows when it decides.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Situation<'a> {
    /// The donor's own score.
    pub own: i32,
    /// The recipient's score as the donor sees it (0 when unknown).
    pub seen: i32,
    /// Whether the donor believes itself, and the recipient, in good standing.
    pub own_good: bool,
    pub their_good: bool,
    /// The group's q tallies, when q strategies play: x and y by score from `lo`.
    pub tallies: Option<&'a Tallies>,
}

impl Strategy {
    pub fn class(self) -> Class {
        match self {
            Strategy::K(_) => Class::K,
            Strategy::H(_) => Class::H,
            Strategy::And { .. } => Class::And,
            Strategy::Or { .. } => Class::Or,
            Strategy::OwnOnly(_) => Class::OwnOnly,
            Strategy::Binary(_) => Class::Binary,
            Strategy::Standing => Class::Standing,
            Strategy::Q(_) => Class::Q,
        }
    }

    /// The strategy's k, for the classes that have one.
    pub fn k(self) -> Option<i8> {
        match self {
            Strategy::K(k)
            | Strategy::Binary(k)
            | Strategy::And { k, .. }
            | Strategy::Or { k, .. } => Some(k),
            _ => None,
        }
    }

    /// Whether the strategy is one of its class's.
    pub fn is_valid(self) -> bool {
        let r = K_MIN..=K_MAX;
        match self {
            Strategy::K(v) | Strategy::H(v) | Strategy::OwnOnly(v) => r.contains(&v),
            Strategy::And { k, h } | Strategy::Or { k, h } => r.contains(&k) && r.contains(&h),
            Strategy::Binary(k) => (-1..=1).contains(&k),
            Strategy::Standing => true,
            Strategy::Q(d) => (1..=99).contains(&d),
        }
    }

    /// Whether the donor helps.
    pub fn helps(self, s: &Situation) -> bool {
        match self {
            Strategy::K(k) | Strategy::Binary(k) => s.seen >= i32::from(k),
            Strategy::H(h) | Strategy::OwnOnly(h) => s.own < i32::from(h),
            Strategy::And { k, h } => s.seen >= i32::from(k) && s.own < i32::from(h),
            Strategy::Or { k, h } => s.seen >= i32::from(k) || s.own < i32::from(h),
            Strategy::Standing => !s.own_good || s.their_good,
            Strategy::Q(d) => s
                .tallies
                .is_some_and(|t| t.gain(s.own) > f64::from(d) / 100.0),
        }
    }

    /// Whether it helps at the start of a generation: own and recipient's
    /// scores 0, both in good standing, the q tallies at their prior. NS98's
    /// "cooperative" (k ≤ 0: "they cooperate with individuals that have not
    /// had an interaction"), for every class.
    pub fn cooperative(self) -> bool {
        let prior = Tallies::new(-5, 5);
        self.helps(&Situation {
            own: 0,
            seen: 0,
            own_good: true,
            their_good: true,
            tallies: Some(&prior),
        })
    }

    /// A short label: "k = 0", "k = 0, h = 1 (AND)", "standing", "Δq = 0.25".
    pub fn label(self) -> String {
        match self {
            Strategy::K(k) => format!("k = {k}"),
            Strategy::H(h) => format!("h = {h}"),
            Strategy::OwnOnly(h) => format!("h = {h} (own score only)"),
            Strategy::And { k, h } => format!("k = {k}, h = {h} (AND)"),
            Strategy::Or { k, h } => format!("k = {k}, h = {h} (OR)"),
            Strategy::Binary(-1) => "cooperator (k = −1)".to_string(),
            Strategy::Binary(0) => "discriminator (k = 0)".to_string(),
            Strategy::Binary(k) => format!("defector (k = {k})"),
            Strategy::Standing => "standing".to_string(),
            Strategy::Q(d) => format!("Δq = {:.2}", f64::from(d) / 100.0),
        }
    }

    /// A number unique to the strategy (the fingerprint's).
    pub fn code(self) -> u64 {
        let v = |x: i8| u64::from(x as u8);
        match self {
            Strategy::K(k) => v(k),
            Strategy::H(h) => 1 << 16 | v(h),
            Strategy::And { k, h } => 2 << 16 | v(k) << 8 | v(h),
            Strategy::Or { k, h } => 3 << 16 | v(k) << 8 | v(h),
            Strategy::OwnOnly(h) => 4 << 16 | v(h),
            Strategy::Binary(k) => 5 << 16 | v(k),
            Strategy::Standing => 6 << 16,
            Strategy::Q(d) => 7 << 16 | u64::from(d),
        }
    }
}

/// LH01's q-strategy estimates for one group: `x[s]` counts rounds in which a
/// recipient with score s or lower was helped, `y[s]` rounds in which one
/// with score s or higher was not, over scores `lo` … `lo + len − 1`; the
/// prior is x = 1, y = 0 for s ≥ 0 and x = 0, y = 1 for s < 0.
#[derive(Clone, Debug, PartialEq)]
pub struct Tallies {
    pub lo: i32,
    pub x: Vec<u32>,
    pub y: Vec<u32>,
}

impl Tallies {
    pub fn new(lo: i32, hi: i32) -> Self {
        let len = (hi - lo + 1) as usize;
        let mut t = Tallies {
            lo,
            x: vec![0; len],
            y: vec![0; len],
        };
        t.reset();
        t
    }

    /// Back to the prior.
    pub fn reset(&mut self) {
        for (i, (x, y)) in self.x.iter_mut().zip(&mut self.y).enumerate() {
            let good = self.lo + i as i32 >= 0;
            (*x, *y) = (u32::from(good), u32::from(!good));
        }
    }

    fn hi(&self) -> i32 {
        self.lo + self.x.len() as i32 - 1
    }

    /// q_s = x_s / (x_s + y_s), the score clamped to the range.
    pub fn q(&self, s: i32) -> f64 {
        let i = (s.clamp(self.lo, self.hi()) - self.lo) as usize;
        f64::from(self.x[i]) / f64::from(self.x[i] + self.y[i])
    }

    /// q_{s+1} − q_{s−1}: the estimated gain from helping at score `s`.
    pub fn gain(&self, s: i32) -> f64 {
        self.q(s + 1) - self.q(s - 1)
    }

    /// Records a round: a recipient with score `score` was helped or not.
    pub fn record(&mut self, score: i32, helped: bool) {
        let i = (score.clamp(self.lo, self.hi()) - self.lo) as usize;
        if helped {
            self.x[i..].iter_mut().for_each(|v| *v += 1);
        } else {
            self.y[..=i].iter_mut().for_each(|v| *v += 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(own: i32, seen: i32) -> Situation<'static> {
        Situation {
            own,
            seen,
            own_good: true,
            their_good: true,
            tallies: None,
        }
    }

    #[test]
    fn k_helps_a_recipient_seen_at_k_or_above() {
        assert!(Strategy::K(0).helps(&at(-5, 0)));
        assert!(!Strategy::K(0).helps(&at(5, -1)));
        assert!(Strategy::K(-5).helps(&at(0, -5)), "−5 always helps");
        assert!(!Strategy::K(6).helps(&at(0, 5)), "+6 never helps");
    }

    #[test]
    fn h_and_own_only_look_only_at_the_donors_score() {
        for s in [Strategy::H(1), Strategy::OwnOnly(1)] {
            assert!(s.helps(&at(0, -5)));
            assert!(!s.helps(&at(1, 5)));
        }
        assert_eq!(Strategy::OwnOnly(1).class(), Class::OwnOnly);
    }

    #[test]
    fn and_needs_both_and_or_either() {
        let and = Strategy::And { k: 0, h: 1 };
        let or = Strategy::Or { k: 0, h: 1 };
        assert!(and.helps(&at(0, 0)));
        assert!(!and.helps(&at(1, 0)) && !and.helps(&at(0, -1)));
        assert!(or.helps(&at(1, 0)) && or.helps(&at(0, -1)));
        assert!(!or.helps(&at(1, -1)));
    }

    #[test]
    fn binary_scorers_are_cooperators_discriminators_and_defectors() {
        let [c, x, d] = [-1, 0, 1].map(Strategy::Binary);
        for seen in [-1, 0] {
            assert!(c.helps(&at(0, seen)) && !d.helps(&at(0, seen)));
        }
        assert!(x.helps(&at(-1, 0)) && !x.helps(&at(0, -1)));
    }

    #[test]
    fn standing_helps_the_good_or_when_itself_bad() {
        let s = |own_good, their_good| {
            Strategy::Standing.helps(&Situation {
                own_good,
                their_good,
                ..at(0, 0)
            })
        };
        assert!(s(true, true) && s(false, true) && s(false, false));
        assert!(!s(true, false));
    }

    #[test]
    fn q_strategies_follow_the_tallies() {
        let mut t = Tallies::new(-5, 5);
        // The prior: q = 1 at s ≥ 0, 0 below; at 0 the gain is 1, at 1 it is 0.
        assert_eq!(
            (t.q(0), t.q(-1), t.gain(0), t.gain(1)),
            (1.0, 0.0, 1.0, 0.0)
        );
        let q = |d, own, t: &Tallies| {
            Strategy::Q(d).helps(&Situation {
                tallies: Some(t),
                ..at(own, 0)
            })
        };
        assert!(q(99, 0, &t) && !q(1, 1, &t) && !q(1, -3, &t));
        // A recipient at −2 is helped: x counts for s ≥ −2.
        t.record(-2, true);
        assert_eq!((t.x[3], t.x[2], t.y[3]), (1, 0, 1));
        assert_eq!(t.q(-2), 0.5);
        // A recipient at 3 is refused: y counts for s ≤ 3.
        t.record(3, false);
        assert_eq!((t.y[8], t.y[9], t.x[8]), (1, 0, 2));
        assert_eq!(t.q(3), 2.0 / 3.0);
        // At the ends the neighbours clamp: at 5, q(6) is q(5).
        assert_eq!(t.gain(5), t.q(5) - t.q(4));
        assert!(!Strategy::Q(50).helps(&at(0, 0)), "no tallies, no help");
        t.reset();
        assert_eq!(t, Tallies::new(-5, 5));
    }

    #[test]
    fn cooperative_means_helping_at_the_start() {
        assert!(Strategy::K(0).cooperative() && !Strategy::K(1).cooperative());
        assert!(Strategy::H(1).cooperative() && !Strategy::H(0).cooperative());
        assert!(Strategy::And { k: 0, h: 1 }.cooperative());
        assert!(!Strategy::And { k: 0, h: 0 }.cooperative());
        assert!(Strategy::Or { k: 1, h: 1 }.cooperative());
        assert!(Strategy::Binary(0).cooperative() && !Strategy::Binary(1).cooperative());
        assert!(Strategy::Standing.cooperative() && Strategy::Q(99).cooperative());
    }

    #[test]
    fn classes_list_their_members_and_the_wire_form_is_stable() {
        let sizes: Vec<usize> = Class::ALL.iter().map(|c| c.members().len()).collect();
        assert_eq!(sizes, [12, 12, 144, 144, 12, 3, 1, 99]);
        for c in Class::ALL {
            assert!(c.members().iter().all(|s| s.is_valid() && s.class() == c));
        }
        assert!(!Strategy::K(7).is_valid() && !Strategy::Q(0).is_valid());
        assert!(!Strategy::Binary(2).is_valid());
        let json = serde_json::to_string(&[
            Strategy::K(0),
            Strategy::And { k: 0, h: 1 },
            Strategy::Standing,
            Strategy::Q(25),
        ])
        .unwrap();
        assert_eq!(
            json,
            r#"[{"k":0},{"and":{"k":0,"h":1}},"standing",{"q":25}]"#
        );
        let codes: std::collections::HashSet<u64> = Class::ALL
            .iter()
            .flat_map(|c| c.members())
            .map(Strategy::code)
            .collect();
        assert_eq!(codes.len(), 12 * 3 + 144 * 2 + 3 + 1 + 99);
        assert_eq!(Strategy::And { k: 0, h: 1 }.label(), "k = 0, h = 1 (AND)");
        assert_eq!(Strategy::Q(25).label(), "Δq = 0.25");
    }
}
