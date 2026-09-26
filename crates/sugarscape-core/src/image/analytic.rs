//! NS98's Methods, computed: the binary-score model of discriminators and
//! defectors (its difference equations, payoffs, the threshold x_min, the
//! stability conditions q > c/b and 1/(1 − w) > (bq + c)/(bq − c)), the
//! equilibrium with unconditional cooperators, and the "universal
//! constant" map with a threshold finder for several starting
//! distributions; and LH01's condition for the standing strategy. Only
//! `+ − × ÷` (no platform `powf`), so results are the same everywhere.

/// NS98's binary model: benefit b, cost c, the chance q that a
/// discriminator knows the recipient's image (and the prior p = 1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Binary {
    pub b: f64,
    pub c: f64,
    pub q: f64,
}

/// The frequencies of discriminators with image 0 and 1, and of defectors
/// with image 0 and 1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct State {
    pub x0: f64,
    pub x1: f64,
    pub y0: f64,
    pub y1: f64,
}

impl State {
    /// A generation's first round: everyone's image is 1 (the payoffs in
    /// round 1, De(1) = bx/2, say so).
    pub fn start(x: f64) -> Self {
        State {
            x0: 0.0,
            x1: x,
            y0: 0.0,
            y1: 1.0 - x,
        }
    }
}

/// `v`^k by repeated multiplication.
fn power(v: f64, k: u32) -> f64 {
    (0..k).fold(1.0, |acc, _| acc * v)
}

impl Binary {
    /// NS98's example (Figs. 1–2): b = 1, c = 0.1, q = 1.
    pub fn ns98() -> Self {
        Binary {
            b: 1.0,
            c: 0.1,
            q: 1.0,
        }
    }

    /// One round of the difference equations: x₀′ = [x₀ + x(1 − φ)q]/2,
    /// x₁′ = [x₁ + x(1 − q + qφ)]/2, y₀′ = [y₀ + y]/2, y₁′ = y₁/2, with
    /// φ = x₁ + y₁.
    pub fn round(&self, s: State) -> State {
        let (x, y, phi, q) = (s.x0 + s.x1, s.y0 + s.y1, s.x1 + s.y1, self.q);
        State {
            x0: (s.x0 + x * (1.0 - phi) * q) / 2.0,
            x1: (s.x1 + x * (1.0 - q + q * phi)) / 2.0,
            y0: (s.y0 + y) / 2.0,
            y1: s.y1 / 2.0,
        }
    }

    /// The round's expected payoffs to discriminators of image 0 and 1 and
    /// to defectors of image 0 and 1.
    pub fn payoffs(&self, s: State) -> [f64; 4] {
        let (b, c, q) = (self.b, self.c, self.q);
        let (x, phi) = (s.x0 + s.x1, s.x1 + s.y1);
        let cost = -c * (1.0 - q + q * phi);
        [
            (cost + b * x * (1.0 - q)) / 2.0,
            (cost + b * x) / 2.0,
            b * x * (1.0 - q) / 2.0,
            b * x / 2.0,
        ]
    }

    /// De(k): a defector's expected payoff in round k (from 1).
    pub fn de(&self, k: u32, x: f64) -> f64 {
        assert!(k >= 1, "rounds count from 1");
        let (b, q) = (self.b, self.q);
        b * x * (1.0 - q + q * power(0.5, k - 1)) / 2.0
    }

    /// Di(k) − De(k), NS98's closed form; at qx = 1 (q = x = 1) its limit
    /// [(b − c) − b·2^−(k−1)]/2.
    pub fn di_minus_de(&self, k: u32, x: f64) -> f64 {
        assert!(k >= 1, "rounds count from 1");
        let (b, c, q) = (self.b, self.c, self.q);
        let qx = 1.0 - q * x;
        if qx == 0.0 {
            return (b - c - b * power(0.5, k - 1)) / 2.0;
        }
        ((1.0 - q) * (b * q * x - c) / qx - b * q * power(0.5, k - 1)
            + q * (b - c) * (1.0 - x) / qx * power((1.0 + q * x) / 2.0, k - 1))
            / 2.0
    }

    /// Di(k) − De(k) by iterating the difference equations and averaging
    /// the payoffs over each type's frequencies.
    pub fn di_minus_de_iterated(&self, k: u32, x: f64) -> f64 {
        assert!(k >= 1, "rounds count from 1");
        let mut s = State::start(x);
        for _ in 1..k {
            s = self.round(s);
        }
        let p = self.payoffs(s);
        let di = (s.x0 * p[0] + s.x1 * p[1]) / (s.x0 + s.x1);
        let de = (s.y0 * p[2] + s.y1 * p[3]) / (s.y0 + s.y1);
        di - de
    }

    /// 2(Di − De) over a random number of rounds (another with probability
    /// w), NS98's closed form; at x = 1 its limit (bq − c)/(1 − w) − 2qb/(2 − w).
    pub fn advantage(&self, w: f64, x: f64) -> f64 {
        let (b, c, q) = (self.b, self.c, self.q);
        if x >= 1.0 {
            return (b * q - c) / (1.0 - w) - 2.0 * q * b / (2.0 - w);
        }
        let qx = 1.0 - q * x;
        (1.0 - q) * (b * q * x - c) / ((1.0 - w) * qx)
            + 2.0 * q * ((b - c) * (1.0 - x) / (qx * (2.0 - w - w * q * x)) - b / (2.0 - w))
    }

    /// 2(Di − De) over exactly `rounds` rounds.
    pub fn advantage_fixed(&self, rounds: u32, x: f64) -> f64 {
        (1..=rounds).map(|k| 2.0 * self.di_minus_de(k, x)).sum()
    }

    /// The threshold x_min in (0, 1) where Di = De (discriminators win
    /// above it), by bisection on `f`; `None` when discriminators lose
    /// even at x = 1.
    fn threshold(f: impl Fn(f64) -> f64) -> Option<f64> {
        if f(1.0) <= 0.0 {
            return None;
        }
        let (mut lo, mut hi) = (0.0, 1.0);
        for _ in 0..200 {
            let mid = (lo + hi) / 2.0;
            if mid == lo || mid == hi {
                break;
            }
            if f(mid) > 0.0 {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        Some(hi)
    }

    /// x_min with random rounds (continuation probability w).
    pub fn x_min(&self, w: f64) -> Option<f64> {
        Self::threshold(|x| self.advantage(w, x))
    }

    /// x_min with exactly `rounds` rounds.
    pub fn x_min_fixed(&self, rounds: u32) -> Option<f64> {
        Self::threshold(|x| self.advantage_fixed(rounds, x))
    }

    /// The fewest mean rounds for discriminators to be stable,
    /// (bq + c)/(bq − c); infinite unless q > c/b.
    pub fn min_rounds(&self) -> f64 {
        let (bq, c) = (self.b * self.q, self.c);
        if bq > c {
            (bq + c) / (bq - c)
        } else {
            f64::INFINITY
        }
    }

    /// Whether discriminators are evolutionarily stable (Di > De at x = 1).
    pub fn stable(&self, w: f64) -> bool {
        self.advantage(w, 1.0) > 0.0
    }

    /// With unconditional cooperators: the discriminator frequency
    /// c(2 − w)/(bwq) below which defectors win.
    pub fn cooperator_equilibrium(&self, w: f64) -> f64 {
        self.c * (2.0 - w) / (self.b * w * self.q)
    }

    /// Dc − De = [−c + bwqx/(2 − w)] / [2(1 − w)].
    pub fn cooperators_over_defectors(&self, w: f64, x: f64) -> f64 {
        (-self.c + self.b * w * self.q * x / (2.0 - w)) / (2.0 * (1.0 - w))
    }

    /// Di − De with discriminators x, defectors y and cooperators z.
    pub fn discriminators_over_defectors(&self, w: f64, x: f64, y: f64, z: f64) -> f64 {
        let (b, c, q) = (self.b, self.c, self.q);
        let qx = 1.0 - q * x;
        (b * q * x - c) * (1.0 - q + q * z) / (2.0 * (1.0 - w) * qx) - b * q * (x + y) / (2.0 - w)
            + q * y * (b - c) / (qx * (2.0 - w - w * q * x))
    }
}

/// A starting distribution for the universal-constant map: a fraction f
/// below 0 and the rest at or above it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Start {
    /// f at −1; the rest at score `rest` (`None`: so high that it never
    /// falls below 0).
    AtMinusOne { rest: Option<u32> },
    /// f spread evenly over −1 … −`below`; the rest at 0.
    Spread { below: u32 },
    /// f spread evenly over −1 … −`below`; the rest evenly over 0 … `above`.
    SpreadBoth { below: u32, above: u32 },
}

/// Score frequencies over `lo …`, plus a mass that never falls below 0.
#[derive(Clone, Debug, PartialEq)]
pub struct Scores {
    pub lo: i64,
    pub x: Vec<f64>,
    pub aloft: f64,
}

impl Scores {
    pub fn new(start: Start, f: f64) -> Self {
        let mut s = Scores {
            lo: 0,
            x: Vec::new(),
            aloft: 0.0,
        };
        match start {
            Start::AtMinusOne { rest } => {
                s.add(-1, f);
                match rest {
                    Some(r) => s.add(i64::from(r), 1.0 - f),
                    None => s.aloft = 1.0 - f,
                }
            }
            Start::Spread { below } => {
                for i in 1..=below {
                    s.add(-i64::from(i), f / f64::from(below));
                }
                s.add(0, 1.0 - f);
            }
            Start::SpreadBoth { below, above } => {
                for i in 1..=below {
                    s.add(-i64::from(i), f / f64::from(below));
                }
                for i in 0..=above {
                    s.add(i64::from(i), (1.0 - f) / f64::from(above + 1));
                }
            }
        }
        s
    }

    fn add(&mut self, score: i64, mass: f64) {
        if self.x.is_empty() {
            self.lo = score;
        }
        while score < self.lo {
            self.x.insert(0, 0.0);
            self.lo -= 1;
        }
        let i = (score - self.lo) as usize;
        if i >= self.x.len() {
            self.x.resize(i + 1, 0.0);
        }
        self.x[i] += mass;
    }

    /// φ: the frequency with score ≥ 0.
    pub fn phi(&self) -> f64 {
        let first = (-self.lo).max(0) as usize;
        self.aloft + self.x.iter().skip(first).sum::<f64>()
    }

    /// One round: xᵢ′ = [xᵢ + xᵢ₋₁φ + xᵢ₊₁(1 − φ)]/2 (everyone k = 0,
    /// unbounded scores). Frequencies below 10⁻⁴⁰ at the ends are dropped.
    pub fn round(&mut self) {
        let phi = self.phi();
        let len = self.x.len();
        let mut next = vec![0.0; len + 2];
        for (i, &v) in self.x.iter().enumerate() {
            next[i + 1] += v / 2.0;
            next[i + 2] += v * phi / 2.0;
            next[i] += v * (1.0 - phi) / 2.0;
        }
        self.lo -= 1;
        let first = next.iter().position(|&v| v > 1e-40).unwrap_or(0);
        let last = next.iter().rposition(|&v| v > 1e-40).unwrap_or(0);
        self.x = next[first..=last].to_vec();
        self.lo += first as i64;
    }
}

/// Where the map goes from a start: all-out cooperation (φ → 1) or
/// defection (after the first round, less than 10⁻¹² of the scores that can
/// still fall at or above 0), within `max_rounds`.
pub fn cooperates(start: Start, f: f64, max_rounds: u32) -> Option<bool> {
    let mut s = Scores::new(start, f);
    for round in 0..max_rounds {
        let phi = s.phi();
        if phi > 1.0 - 1e-12 {
            return Some(true);
        }
        if (round > 0 || s.aloft == 0.0) && phi - s.aloft < 1e-12 {
            return Some(false);
        }
        s.round();
    }
    None
}

/// The largest fraction below 0 from which the map still reaches all-out
/// cooperation (NS98: "0.7380294688360…", start unstated), by bisection;
/// an undecided run counts as not cooperating.
pub fn universal_threshold(start: Start) -> f64 {
    let (mut lo, mut hi) = (0.0, 1.0);
    for _ in 0..45 {
        let mid = (lo + hi) / 2.0;
        if cooperates(start, mid, 1_000_000) == Some(true) {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    lo
}

/// LH01's r = (m − 1)/(n + m − 1): the expected rounds as a recipient
/// before next being a donor or the game's end (random rounds, mean m;
/// a generation plays at least one, so m ≥ 1).
pub fn standing_r(n: u32, m: u32) -> f64 {
    assert!(m >= 1, "a generation plays at least one round");
    f64::from(m - 1) / f64::from(n + m - 1)
}

/// LH01's v = ε/(e + ε): the chance a perceived refusal was misperceived.
pub fn standing_v(e: f64, eps: f64) -> f64 {
    if e + eps > 0.0 {
        eps / (e + eps)
    } else {
        0.0
    }
}

/// LH01's condition for standing to be a strict best reply to itself:
/// vrb < c < rb (with no perception errors, v = 0: rb − c > 0).
pub fn standing_stable(b: f64, c: f64, n: u32, m: u32, e: f64, eps: f64) -> bool {
    let r = standing_r(n, m);
    let v = standing_v(e, eps);
    v * r * b < c && c < r * b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "rounds count from 1")]
    fn de_has_no_round_zero() {
        Binary::ns98().de(0, 0.5);
    }

    #[test]
    #[should_panic(expected = "rounds count from 1")]
    fn di_minus_de_has_no_round_zero() {
        Binary::ns98().di_minus_de(0, 0.5);
    }

    #[test]
    #[should_panic(expected = "rounds count from 1")]
    fn the_iterated_difference_has_no_round_zero() {
        Binary::ns98().di_minus_de_iterated(0, 0.5);
    }

    #[test]
    #[should_panic(expected = "at least one round")]
    fn standing_r_needs_a_round() {
        standing_r(100, 0);
    }

    #[test]
    fn standing_r_is_zero_with_one_round() {
        assert_eq!(standing_r(100, 1), 0.0);
    }

    fn close(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() <= tol
    }

    #[test]
    fn the_difference_equations_conserve_each_type_and_match_the_closed_forms() {
        for m in [
            Binary::ns98(),
            Binary {
                b: 1.0,
                c: 0.3,
                q: 0.6,
            },
        ] {
            for x in [0.1, 0.5, 0.9] {
                let mut s = State::start(x);
                for _ in 0..10 {
                    s = m.round(s);
                    assert!(close(s.x0 + s.x1, x, 1e-15) && close(s.y0 + s.y1, 1.0 - x, 1e-15));
                }
                for k in 1..=12 {
                    let a = m.di_minus_de(k, x);
                    let b = m.di_minus_de_iterated(k, x);
                    assert!(close(a, b, 1e-14), "{m:?} x {x} k {k}: {a} vs {b}");
                }
                // Round 1: De = bx/2, and a discriminator pays c half the time.
                assert!(close(m.de(1, x), m.b * x / 2.0, 1e-15));
                assert!(close(m.di_minus_de(1, x), -m.c / 2.0, 1e-15));
            }
        }
    }

    #[test]
    fn the_random_rounds_total_sums_the_rounds() {
        let m = Binary {
            b: 1.0,
            c: 0.2,
            q: 0.8,
        };
        for (w, x) in [(0.5, 0.3), (0.9, 0.7), (0.95, 0.99)] {
            let mut sum = 0.0;
            let mut wk = 1.0;
            for k in 1..=3000 {
                sum += 2.0 * wk * m.di_minus_de(k, x);
                wk *= w;
            }
            assert!(close(sum, m.advantage(w, x), 1e-12), "w {w} x {x}");
        }
        // The x = 1 limit agrees with x just below it.
        assert!(close(
            m.advantage(0.9, 1.0),
            m.advantage(0.9, 1.0 - 1e-9),
            1e-6
        ));
    }

    #[test]
    fn stability_needs_q_above_c_over_b_and_about_1_2_rounds() {
        let m = Binary::ns98();
        assert!(close(m.min_rounds(), 1.1 / 0.9, 1e-15));
        // 1/(1 − w) = 1.2 rounds falls just short; 1.25 is enough.
        assert!(!m.stable(1.0 - 1.0 / 1.2) && m.stable(1.0 - 1.0 / 1.25));
        let w_star = 1.0 - 1.0 / m.min_rounds();
        assert!(!m.stable(w_star - 1e-9) && m.stable(w_star + 1e-9));
        for q in [0.05, 0.1] {
            let m = Binary { q, ..m };
            assert_eq!(m.min_rounds(), f64::INFINITY);
            assert!(!m.stable(0.999_999));
        }
        let m = Binary { q: 0.2, ..m };
        assert!(close(m.min_rounds(), 3.0, 1e-12) && m.stable(1.0 - 1.0 / 3.1));
        // x_min falls below 1 exactly when stable, with q = 1 at c(2 − w)/(bw).
        let m = Binary::ns98();
        assert_eq!(m.x_min(1.0 - 1.0 / 1.2), None);
        let x = m.x_min(0.5).unwrap();
        assert!(close(x, 0.1 * 1.5 / 0.5, 1e-12), "{x}");
        assert!(close(m.advantage(0.5, x), 0.0, 1e-12));
    }

    #[test]
    fn cooperators_hold_even_with_defectors_at_c_2_minus_w_over_bwq() {
        let m = Binary {
            b: 1.0,
            c: 0.1,
            q: 0.9,
        };
        for w in [0.5, 0.8, 0.95] {
            let x = m.cooperator_equilibrium(w);
            assert!(close(m.cooperators_over_defectors(w, x), 0.0, 1e-15));
            assert!(m.cooperators_over_defectors(w, x * 1.01) > 0.0);
            assert!(m.cooperators_over_defectors(w, x * 0.99) < 0.0);
        }
        // Without cooperators the three-type formula is half the two-type one.
        for (w, x) in [(0.6, 0.4), (0.9, 0.8)] {
            let three = m.discriminators_over_defectors(w, x, 1.0 - x, 0.0);
            assert!(close(2.0 * three, m.advantage(w, x), 1e-12));
        }
    }

    #[test]
    fn fixed_rounds_thresholds_fall_with_more_rounds() {
        let m = Binary::ns98();
        assert_eq!(m.x_min_fixed(1), None, "one round: helping only costs");
        let xs: Vec<f64> = [2, 3, 5, 10]
            .iter()
            .map(|&k| m.x_min_fixed(k).unwrap())
            .collect();
        assert!(xs.windows(2).all(|p| p[1] < p[0]), "{xs:?}");
    }

    #[test]
    fn the_universal_map_conserves_mass_and_its_ends_are_absorbing() {
        let mut s = Scores::new(Start::Spread { below: 3 }, 0.4);
        for _ in 0..50 {
            s.round();
            assert!(close(s.x.iter().sum::<f64>(), 1.0, 1e-12));
        }
        assert_eq!(
            cooperates(Start::AtMinusOne { rest: Some(0) }, 0.0, 10),
            Some(true)
        );
        assert_eq!(
            cooperates(Start::AtMinusOne { rest: Some(0) }, 1.0, 10),
            Some(false)
        );
        assert_eq!(
            cooperates(Start::AtMinusOne { rest: Some(0) }, 0.49, 100_000),
            Some(true)
        );
        assert_eq!(
            cooperates(Start::AtMinusOne { rest: Some(0) }, 0.51, 100_000),
            Some(false)
        );
        let s = Scores::new(Start::AtMinusOne { rest: None }, 0.3);
        assert!(close(s.phi(), 0.7, 1e-15) && s.x == [0.3]);
    }

    #[test]
    fn standing_is_a_best_reply_when_vrb_is_below_c_below_rb() {
        // LH01 Fig. 4b: e = ε = 0.025 gives v = 0.5, r = 0.833 — not met at c = 0.25.
        assert!(close(standing_r(100, 500), 499.0 / 599.0, 1e-15));
        assert_eq!(standing_v(0.025, 0.025), 0.5);
        assert!(!standing_stable(1.0, 0.25, 100, 500, 0.025, 0.025));
        // e = 0.04, ε = 0.01 (v = 0.2) meets it; so does ε = 0.
        assert!(standing_stable(1.0, 0.25, 100, 500, 0.04, 0.01));
        assert!(standing_stable(1.0, 0.25, 100, 500, 0.05, 0.0));
        assert!(
            !standing_stable(1.0, 0.9, 100, 500, 0.05, 0.0),
            "c above rb"
        );
    }
}
