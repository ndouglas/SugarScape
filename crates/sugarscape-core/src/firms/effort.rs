//! An agent's best effort, and the analytics of Axtell's §2.
//!
//! Output is O(E) = a·E + b·E^β over a firm's total effort E; income is an
//! equal share (or a seniority share, or base pay plus a bonus); utility is
//! Cobb–Douglas, (income)^θ (1 − e)^(1−θ), or CES. For the base case (β = 2,
//! Cobb–Douglas, shares proportional to output) the optimum is A99's
//! closed form (5); for constant returns it is (6); otherwise a bisection on
//! the first-order condition (Cobb–Douglas) or a bracketed search (base pay,
//! CES). Non-integer powers go through the portable ln and exp.

use crate::portable::{exp_neg, ln};

/// x^y for x ≥ 0, bit-for-bit the same on every platform.
pub fn powf(x: f64, y: f64) -> f64 {
    if x <= 0.0 {
        return if y > 0.0 {
            0.0
        } else if y == 0.0 {
            1.0
        } else {
            f64::INFINITY
        };
    }
    if y == 0.0 || x == 1.0 {
        return 1.0;
    }
    if y == 1.0 {
        return x;
    }
    if y == 2.0 {
        return x * x;
    }
    let z = y * ln(x);
    if z <= 0.0 {
        exp_neg(z)
    } else if z > 708.0 {
        f64::INFINITY
    } else {
        1.0 / exp_neg(-z)
    }
}

/// A firm's technology.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tech {
    pub a: f64,
    pub b: f64,
    pub beta: f64,
}

impl Tech {
    pub fn output(&self, e: f64) -> f64 {
        let e = e.max(0.0);
        self.a * e + self.b * powf(e, self.beta)
    }
}

/// An agent's preferences between income and leisure.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Prefs {
    /// (income)^θ (leisure)^(1−θ).
    CobbDouglas { theta: f64 },
    /// CES with weight δ on income; `minus` is the text's (−ρ) convention.
    Ces { delta: f64, rho: f64, minus: bool },
}

impl Prefs {
    /// The weight on income (θ, or δ for CES): what hiring standards compare.
    pub fn weight(&self) -> f64 {
        match *self {
            Prefs::CobbDouglas { theta } => theta,
            Prefs::Ces { delta, .. } => delta,
        }
    }

    pub fn utility(&self, income: f64, leisure: f64) -> f64 {
        let (y, l) = (income.max(0.0), leisure.clamp(0.0, 1.0));
        match *self {
            Prefs::CobbDouglas { theta } => powf(y, theta) * powf(l, 1.0 - theta),
            Prefs::Ces { delta, rho, minus } => {
                let r = if minus { -rho } else { rho };
                if r.abs() < 1e-9 {
                    return powf(y, delta) * powf(l, 1.0 - delta);
                }
                let term = |x: f64| {
                    if x <= 0.0 {
                        if r > 0.0 {
                            0.0
                        } else {
                            f64::INFINITY
                        }
                    } else {
                        powf(x, r)
                    }
                };
                let s = delta * term(y) + (1.0 - delta) * term(l);
                if s.is_infinite() {
                    return 0.0;
                }
                powf(s, 1.0 / r)
            }
        }
    }
}

/// How a firm pays the agent choosing its effort.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Share {
    /// A fixed fraction of output (1/N for equal shares, a seniority weight).
    Fraction(f64),
    /// Base pay `own` plus an equal share of max(0, O − `own` − `others`)
    /// among `n` members.
    Base { own: f64, others: f64, n: f64 },
}

impl Share {
    pub fn income(&self, output: f64) -> f64 {
        match *self {
            Share::Fraction(w) => w * output,
            Share::Base { own, others, n } => own + ((output - own - others) / n).max(0.0),
        }
    }
}

/// One option an agent weighs: a firm's technology, the others' effort, and
/// its share.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Choice {
    pub tech: Tech,
    pub others: f64,
    pub share: Share,
}

impl Choice {
    pub fn utility(&self, prefs: &Prefs, e: f64) -> f64 {
        prefs.utility(
            self.share.income(self.tech.output(e + self.others)),
            1.0 - e,
        )
    }
}

/// How the effort is searched.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Search {
    Exact,
    Grid(u32),
}

/// A99 (5): the Cobb–Douglas optimum for β = 2, b > 0, equal shares,
/// given the others' effort `others`.
pub fn closed_form(theta: f64, others: f64, a: f64, b: f64) -> f64 {
    let t = theta;
    let x = 1.0 + others;
    let disc = a * a + 4.0 * a * b * t * t * x + 4.0 * b * b * t * t * x * x;
    let e = (-a - 2.0 * b * (others - t) + disc.sqrt()) / (2.0 * b * (1.0 + t));
    e.clamp(0.0, 1.0)
}

/// The best effort in [lo, hi] and its utility.
pub fn best(prefs: &Prefs, choice: &Choice, lo: f64, hi: f64, search: Search) -> (f64, f64) {
    let (lo, hi) = (lo.clamp(0.0, 1.0), hi.clamp(0.0, 1.0));
    let e = match search {
        Search::Grid(steps) => {
            let steps = steps.max(1);
            let mut top = (lo, choice.utility(prefs, lo));
            for k in 0..=steps {
                let e = f64::from(k) / f64::from(steps);
                if e < lo || e > hi {
                    continue;
                }
                let u = choice.utility(prefs, e);
                if u > top.1 {
                    top = (e, u);
                }
            }
            return top;
        }
        Search::Exact => exact(prefs, choice, lo, hi),
    };
    (e, choice.utility(prefs, e))
}

fn exact(prefs: &Prefs, choice: &Choice, lo: f64, hi: f64) -> f64 {
    let t = choice.tech;
    match (prefs, choice.share) {
        (Prefs::CobbDouglas { theta }, Share::Fraction(_)) => {
            // Shares proportional to output: the optimum ignores the share.
            if t.beta == 2.0 && t.b > 0.0 {
                closed_form(*theta, choice.others, t.a, t.b).clamp(lo, hi)
            } else if t.b == 0.0 || t.beta == 1.0 {
                // A99 (6): constant returns.
                (theta - choice.others * (1.0 - theta)).clamp(lo, hi)
            } else {
                bisect_cd(*theta, &t, choice.others, lo, hi)
            }
        }
        _ => bracket(prefs, choice, lo, hi),
    }
}

/// The root of d/de ln U = θ O′(E)/O(E) − (1−θ)/(1−e) in [lo, hi]
/// (decreasing in e): safeguarded Newton, falling back to bisection.
fn bisect_cd(theta: f64, t: &Tech, others: f64, lo: f64, hi: f64) -> f64 {
    // g and g′ from one power: E^β (E^(β−1) = E^β/E, E^(β−2) = E^β/E²).
    let g = |e: f64| -> (f64, f64) {
        let big = e + others;
        let cost = if e >= 1.0 {
            if theta < 1.0 {
                f64::INFINITY
            } else {
                0.0
            }
        } else {
            (1.0 - theta) / (1.0 - e)
        };
        let dcost = if e >= 1.0 {
            f64::INFINITY
        } else {
            (1.0 - theta) / ((1.0 - e) * (1.0 - e))
        };
        if big <= 0.0 {
            return (
                if theta > 0.0 { f64::INFINITY } else { -cost },
                f64::NEG_INFINITY,
            );
        }
        let p = powf(big, t.beta);
        let o = t.a * big + t.b * p;
        let o1 = t.a + t.b * t.beta * p / big;
        let o2 = t.b * t.beta * (t.beta - 1.0) * p / (big * big);
        let gain = theta * o1 / o;
        let dgain = theta * (o2 * o - o1 * o1) / (o * o);
        (gain - cost, dgain - dcost)
    };
    if g(lo).0 <= 0.0 {
        return lo;
    }
    if g(hi).0 >= 0.0 {
        return hi;
    }
    let (mut l, mut h) = (lo, hi);
    let mut x = 0.5 * (l + h);
    for _ in 0..100 {
        let (v, d) = g(x);
        if v > 0.0 {
            l = x;
        } else {
            h = x;
        }
        let newton = x - v / d;
        let next = if d < 0.0 && newton > l && newton < h {
            newton
        } else {
            0.5 * (l + h)
        };
        if (next - x).abs() <= 1e-14 || h - l <= 1e-14 {
            return next;
        }
        x = next;
    }
    x
}

/// Where `f` (f(lo) > 0 > f(hi)) crosses zero: the Illinois variant of
/// regula falsi, which keeps the bracket and converges superlinearly.
fn root(f: impl Fn(f64) -> f64, lo: f64, hi: f64) -> f64 {
    let (mut a, mut b) = (lo, hi);
    let (mut fa, mut fb) = (f(a), f(b));
    if !fa.is_finite() || !fb.is_finite() {
        // Infinite ends: halve until both are finite, keeping the signs.
        for _ in 0..60 {
            let m = 0.5 * (a + b);
            let fm = f(m);
            if fm > 0.0 {
                a = m;
                fa = fm;
            } else {
                b = m;
                fb = fm;
            }
            if fa.is_finite() && fb.is_finite() {
                break;
            }
        }
        if !fa.is_finite() || !fb.is_finite() {
            return 0.5 * (a + b);
        }
    }
    let mut side = 0;
    for _ in 0..100 {
        if b - a <= 1e-13 {
            break;
        }
        // A false-position step, or a bisection when extreme values push it
        // onto the bracket's edge.
        let mut c = (a * fb - b * fa) / (fb - fa);
        if !(c > a && c < b) {
            c = 0.5 * (a + b);
            side = 0;
        }
        let fc = f(c);
        if fc == 0.0 {
            return c;
        }
        if fc > 0.0 {
            a = c;
            fa = fc;
            if side == 1 {
                fb *= 0.5;
            }
            side = 1;
        } else {
            b = c;
            fb = fc;
            if side == -1 {
                fa *= 0.5;
            }
            side = -1;
        }
        if (b - a).abs() <= 1e-13 {
            break;
        }
    }
    0.5 * (a + b)
}

/// The best of `candidates` by utility.
fn best_of(prefs: &Prefs, choice: &Choice, candidates: &[f64]) -> f64 {
    let mut top = (candidates[0], choice.utility(prefs, candidates[0]));
    for &e in &candidates[1..] {
        let u = choice.utility(prefs, e);
        if u > top.1 {
            top = (e, u);
        }
    }
    top.0
}

/// Base pay or CES: the maximum on [lo, hi], split where the bonus starts
/// (income is kinked there). Cobb–Douglas solves its first-order condition
/// analytically in each piece; CES by the sign of its utility's slope (a
/// central difference). Endpoints are always candidates.
fn bracket(prefs: &Prefs, choice: &Choice, lo: f64, hi: f64) -> f64 {
    if hi <= lo {
        return lo;
    }
    let t = choice.tech;
    let mut cuts = vec![lo];
    if let Share::Base { own, others, .. } = choice.share {
        let floor = own + others;
        let over = |e: f64| t.output(e + choice.others) - floor;
        if over(lo) < 0.0 && over(hi) > 0.0 {
            cuts.push(root(|e| -over(e), lo, hi));
        }
    }
    cuts.push(hi);
    let mut candidates = cuts.clone();
    // Whether income rises with output on a piece: always for proportional
    // shares; for base pay, above the kink (decided per piece, not per
    // point, so rounding at the kink cannot hide a piece's slope).
    let floor = match choice.share {
        Share::Base { own, others, .. } => own + others,
        Share::Fraction(_) => f64::NEG_INFINITY,
    };
    for w in cuts.windows(2) {
        let (a, b) = (w[0], w[1]);
        if b - a <= 1e-12 {
            continue;
        }
        let bonus = t.output(0.5 * (a + b) + choice.others) >= floor;
        let slope: Box<dyn Fn(f64) -> f64> = match (prefs, choice.share) {
            (Prefs::CobbDouglas { theta }, Share::Base { n, .. }) => {
                let theta = *theta;
                Box::new(move |e: f64| {
                    let big = e + choice.others;
                    let y = choice.share.income(t.output(big));
                    let dy = if bonus {
                        let p = if big > 0.0 { powf(big, t.beta) } else { 0.0 };
                        (t.a + if big > 0.0 {
                            t.b * t.beta * p / big
                        } else {
                            0.0
                        }) / n
                    } else {
                        0.0
                    };
                    let gain = if y > 0.0 {
                        theta * dy / y
                    } else if dy > 0.0 && theta > 0.0 {
                        f64::INFINITY
                    } else {
                        0.0
                    };
                    let cost = if e >= 1.0 {
                        f64::INFINITY
                    } else {
                        (1.0 - theta) / (1.0 - e)
                    };
                    gain - cost
                })
            }
            (Prefs::Ces { delta, rho, minus }, share) => {
                // The sign of dU/de: δ y^(r−1) y′ − (1−δ) l^(r−1), with r the
                // exponent in the convention used (CES utility rises with its
                // inner sum when r > 0 and falls when r < 0; the r factor cancels).
                let (delta, r) = (*delta, if *minus { -*rho } else { *rho });
                Box::new(move |e: f64| {
                    let big = e + choice.others;
                    let o = t.output(big);
                    let y = share.income(o);
                    let o1 = if big > 0.0 {
                        t.a + t.b * t.beta * powf(big, t.beta) / big
                    } else {
                        t.a
                    };
                    let dy = match share {
                        Share::Fraction(w) => w * o1,
                        Share::Base { n, .. } => {
                            if bonus {
                                o1 / n
                            } else {
                                0.0
                            }
                        }
                    };
                    let l = 1.0 - e;
                    if r.abs() < 1e-9 {
                        let gain = if y > 0.0 {
                            delta * dy / y
                        } else if dy > 0.0 {
                            f64::INFINITY
                        } else {
                            0.0
                        };
                        return gain
                            - if l > 0.0 {
                                (1.0 - delta) / l
                            } else {
                                f64::INFINITY
                            };
                    }
                    let py = if y > 0.0 {
                        powf(y, r - 1.0)
                    } else if r < 1.0 {
                        f64::INFINITY
                    } else {
                        0.0
                    };
                    let pl = if l > 0.0 {
                        powf(l, r - 1.0)
                    } else if r < 1.0 {
                        f64::INFINITY
                    } else {
                        0.0
                    };
                    let (g, c) = (delta * py * dy, (1.0 - delta) * pl);
                    if g.is_infinite() && c.is_infinite() {
                        0.0
                    } else {
                        g - c
                    }
                })
            }
            (Prefs::CobbDouglas { .. }, _) => {
                unreachable!("Cobb–Douglas with proportional shares is solved earlier")
            }
        };
        match prefs {
            Prefs::CobbDouglas { .. } => {
                // Concave in e; a single sign change is the whole story
                // (verified against a fine grid; left as-is).
                if slope(a) > 0.0 && slope(b) < 0.0 {
                    candidates.push(root(&slope, a, b));
                }
            }
            Prefs::Ces { .. } => {
                // Not always concave: scan for every + → − sign change in
                // the piece and refine each with root(), so an interior
                // maximum between two same-signed endpoints isn't missed.
                const SCAN: usize = 32;
                let mut prev_x = a;
                let mut prev_s = slope(a);
                for k in 1..=SCAN {
                    let x = a + (b - a) * (k as f64) / (SCAN as f64);
                    let s = slope(x);
                    if prev_s > 0.0 && s < 0.0 {
                        candidates.push(root(&slope, prev_x, x));
                    }
                    prev_x = x;
                    prev_s = s;
                }
            }
        }
    }
    best_of(prefs, choice, &candidates)
}

/// §2: the symmetric Nash effort in a homogeneous group of `n` agents of
/// preference θ (equal shares, β = 2), and each agent's utility there.
pub fn nash(theta: f64, n: u32, a: f64, b: f64) -> (f64, f64) {
    let others = f64::from(n - 1);
    let (mut l, mut h) = (0.0, 1.0);
    for _ in 0..80 {
        let m = 0.5 * (l + h);
        if closed_form(theta, others * m, a, b) > m {
            l = m;
        } else {
            h = m;
        }
    }
    let e = 0.5 * (l + h);
    let big = f64::from(n) * e;
    let u = Prefs::CobbDouglas { theta }.utility((a * big + b * big * big) / f64::from(n), 1.0 - e);
    (e, u)
}

/// A99 (9): the Jacobian's off-diagonal entry for an agent of preference θ
/// against others' effort `others` (β = 2).
pub fn jacobian_k(theta: f64, others: f64, a: f64, b: f64) -> f64 {
    let x = 1.0 + others;
    let root = (a * a + 4.0 * b * theta * theta * x * (a + b * x)).sqrt();
    (-1.0 + theta * theta * (a + 2.0 * b * x) / root) / (1.0 + theta)
}

/// The dominant eigenvalue (N − 1)·k of a homogeneous group at its Nash
/// equilibrium; the group is stable while it is at least −1.
pub fn eigenvalue(theta: f64, n: u32, a: f64, b: f64) -> f64 {
    if n < 2 {
        return 0.0;
    }
    let (e, _) = nash(theta, n, a, b);
    f64::from(n - 1) * jacobian_k(theta, f64::from(n - 1) * e, a, b)
}

/// The largest stable homogeneous group of preference θ (up to `cap`).
pub fn max_stable_size(theta: f64, a: f64, b: f64, cap: u32) -> u32 {
    (1..=cap)
        .take_while(|&n| eigenvalue(theta, n, a, b) >= -1.0)
        .last()
        .unwrap_or(1)
}

/// The homogeneous group size that maximizes each member's Nash utility.
pub fn optimal_size(theta: f64, a: f64, b: f64, cap: u32) -> u32 {
    let mut top = (1, f64::MIN);
    for n in 1..=cap {
        let u = nash(theta, n, a, b).1;
        if u > top.1 {
            top = (n, u);
        }
    }
    top.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cd(theta: f64) -> Prefs {
        Prefs::CobbDouglas { theta }
    }

    fn equal(n: f64, others: f64) -> Choice {
        Choice {
            tech: Tech {
                a: 1.0,
                b: 1.0,
                beta: 2.0,
            },
            others,
            share: Share::Fraction(1.0 / n),
        }
    }

    #[test]
    fn the_closed_form_is_the_line_searchs_limit() {
        for &theta in &[0.1, 0.5, 0.7, 0.95] {
            for &others in &[0.0, 0.5, 2.0, 10.0] {
                let c = equal(3.0, others);
                let (e, _) = best(&cd(theta), &c, 0.0, 1.0, Search::Exact);
                let (g, _) = best(&cd(theta), &c, 0.0, 1.0, Search::Grid(100_000));
                assert!((e - g).abs() < 2e-5, "θ {theta} E {others}: {e} vs {g}");
                // And the bisection agrees with (5) at β = 2.
                let b = bisect_cd(theta, &c.tech, others, 0.0, 1.0);
                assert!((e - b).abs() < 1e-9, "{e} vs {b}");
            }
        }
    }

    #[test]
    fn table_1_reproduces_to_three_places() {
        // A99 Table 1, θ = 0.7: e*, U(e*), k and (N − 1)k for N = 1..7.
        let rows = [
            (1, 0.770, 0.799, f64::NAN),
            (2, 0.646, 0.964, -0.188),
            (3, 0.558, 1.036, -0.368),
            (4, 0.492, 1.065, -0.547),
            (5, 0.441, 1.069, -0.726),
            (6, 0.399, 1.061, -0.904),
            (7, 0.364, 1.045, -1.082),
        ];
        for (n, e, u, lambda) in rows {
            let (ne, nu) = nash(0.7, n, 1.0, 1.0);
            assert!(
                (ne - e).abs() < 5e-4 && (nu - u).abs() < 5e-4,
                "N {n}: {ne} {nu}"
            );
            if n > 1 {
                assert!(
                    (eigenvalue(0.7, n, 1.0, 1.0) - lambda).abs() < 5e-4,
                    "N {n}"
                );
            }
        }
        assert_eq!(
            (
                max_stable_size(0.7, 1.0, 1.0, 50),
                optimal_size(0.7, 1.0, 1.0, 50)
            ),
            (6, 5)
        );
    }

    #[test]
    fn optimal_and_stable_sizes_grow_with_theta() {
        let sizes: Vec<(u32, u32)> = [0.5, 0.8, 0.9, 0.95]
            .iter()
            .map(|&t| {
                (
                    optimal_size(t, 1.0, 1.0, 200),
                    max_stable_size(t, 1.0, 1.0, 200),
                )
            })
            .collect();
        assert_eq!(sizes, [(2, 3), (8, 9), (18, 19), (38, 39)]);
    }

    #[test]
    fn constant_returns_follow_equation_6() {
        let c = Choice {
            tech: Tech {
                a: 1.0,
                b: 0.0,
                beta: 2.0,
            },
            others: 0.5,
            share: Share::Fraction(0.5),
        };
        let (e, _) = best(&cd(0.6), &c, 0.0, 1.0, Search::Exact);
        assert!((e - (0.6 - 0.5 * 0.4)).abs() < 1e-12);
        let (g, _) = best(&cd(0.6), &c, 0.0, 1.0, Search::Grid(100_000));
        assert!((e - g).abs() < 2e-5);
    }

    #[test]
    fn general_beta_ces_and_base_pay_find_the_maximum() {
        let prefs = [
            cd(0.6),
            Prefs::Ces {
                delta: 0.6,
                rho: 0.5,
                minus: true,
            },
            Prefs::Ces {
                delta: 0.4,
                rho: -0.5,
                minus: false,
            },
        ];
        let choices = [
            Choice {
                tech: Tech {
                    a: 1.0,
                    b: 1.0,
                    beta: 1.7,
                },
                others: 1.2,
                share: Share::Fraction(0.25),
            },
            Choice {
                tech: Tech {
                    a: 0.3,
                    b: 1.1,
                    beta: 2.1,
                },
                others: 0.0,
                share: Share::Fraction(1.0),
            },
            Choice {
                tech: Tech {
                    a: 1.0,
                    b: 1.0,
                    beta: 2.0,
                },
                others: 2.0,
                share: Share::Base {
                    own: 0.4,
                    others: 1.0,
                    n: 4.0,
                },
            },
        ];
        for p in &prefs {
            for c in &choices {
                let (e, u) = best(p, c, 0.0, 1.0, Search::Exact);
                let (_, ug) = best(p, c, 0.0, 1.0, Search::Grid(20_000));
                assert!(u >= ug - 1e-6, "{p:?} {c:?}: {e} {u} < {ug}");
            }
        }
    }

    #[test]
    fn the_solvers_match_a_fine_grid_on_random_cases() {
        use crate::rng;
        use rand::Rng;
        let mut r = rng::seeded(11);
        for case in 0..600 {
            let prefs = if case % 2 == 0 {
                Prefs::CobbDouglas { theta: r.gen() }
            } else {
                Prefs::Ces {
                    delta: r.gen(),
                    rho: r.gen_range(-1.0..10.0),
                    minus: r.gen(),
                }
            };
            let tech = Tech {
                a: r.gen_range(0.0..1.0),
                b: r.gen_range(0.5..1.5),
                beta: r.gen_range(1.5..2.1),
            };
            let n: f64 = r.gen_range(1.0..20.0_f64).floor();
            let share = if r.gen::<bool>() {
                Share::Fraction(1.0 / n)
            } else {
                let own = r.gen_range(0.0..1.0);
                Share::Base {
                    own,
                    others: own * (n - 1.0) * r.gen_range(0.5..1.5),
                    n,
                }
            };
            let choice = Choice {
                tech,
                others: r.gen_range(0.0..(n - 1.0) * 0.8 + 0.01),
                share,
            };
            let (lo, hi) = if r.gen::<bool>() {
                (0.0, 1.0)
            } else {
                let c: f64 = r.gen();
                (c - 0.05, c + 0.05)
            };
            let (e, u) = best(&prefs, &choice, lo, hi, Search::Exact);
            // (Checked once at 20 000 cases against a 20 000-step grid.)
            let (g, ug) = best(&prefs, &choice, lo, hi, Search::Grid(5_000));
            assert!(
                u >= ug - 1e-7 * ug.abs().max(1.0),
                "case {case}: {prefs:?} {choice:?} [{lo}, {hi}]: exact {e} {u} < grid {g} {ug}"
            );
        }
    }

    #[test]
    fn a_window_clamps_the_optimum() {
        let c = equal(2.0, 0.3);
        let (free, _) = best(&cd(0.9), &c, 0.0, 1.0, Search::Exact);
        let (held, _) = best(&cd(0.9), &c, 0.1, 0.2, Search::Exact);
        assert!(free > 0.2);
        assert_eq!(held, 0.2);
    }

    #[test]
    fn ces_approaches_its_limits() {
        // The text's convention: ρ = −1 is linear, ρ → 0 Cobb–Douglas.
        let linear = Prefs::Ces {
            delta: 0.3,
            rho: -1.0,
            minus: true,
        };
        assert!((linear.utility(2.0, 0.5) - (0.3 * 2.0 + 0.7 * 0.5)).abs() < 1e-12);
        let near = Prefs::Ces {
            delta: 0.3,
            rho: 1e-12,
            minus: true,
        };
        assert!((near.utility(2.0, 0.5) - cd(0.3).utility(2.0, 0.5)).abs() < 1e-9);
        // Large ρ: Leontief, near the smaller input.
        let leontief = Prefs::Ces {
            delta: 0.5,
            rho: 200.0,
            minus: true,
        };
        assert!((leontief.utility(2.0, 0.5) - 0.5).abs() < 0.01);
    }

    #[test]
    fn powf_matches_the_library() {
        for &(x, y) in &[(0.3, 0.7), (2.5, 1.9), (10.0, 2.1), (0.01, 3.0)] {
            let (p, q) = (powf(x, y), x.powf(y));
            assert!(((p - q) / q).abs() < 1e-12, "{x}^{y}: {p} {q}");
        }
        assert_eq!(
            (powf(0.0, 0.5), powf(0.0, 0.0), powf(3.0, 2.0)),
            (0.0, 1.0, 9.0)
        );
    }

    #[test]
    fn ces_finds_interior_maxima_the_bracket_could_miss() {
        // A printed-sign CES case (ρ = 0.706, δ = 0.67) where a single
        // sign-change check per piece missed an interior maximum and
        // returned the boundary e = 0 instead.
        let prefs = Prefs::Ces {
            delta: 0.67,
            rho: 0.706,
            minus: false,
        };
        let choice = Choice {
            tech: Tech {
                a: 0.0,
                b: 1.4,
                beta: 1.0,
            },
            others: 0.0,
            share: Share::Fraction(0.25),
        };
        let (e, u) = best(&prefs, &choice, 0.0, 1.0, Search::Exact);
        let (_, ug) = best(&prefs, &choice, 0.0, 1.0, Search::Grid(20_000));
        assert!(u >= ug - 1e-6, "exact {e} {u} < grid {ug}");
    }

    #[test]
    fn ces_with_rho_in_0_1_matches_a_fine_grid_under_printed_sign_and_base_pay() {
        use crate::rng;
        use rand::Rng;
        let mut r = rng::seeded(37);
        for case in 0..400 {
            let delta: f64 = r.gen();
            let rho: f64 = r.gen_range(0.0..1.0);
            let prefs = Prefs::Ces {
                delta,
                rho,
                minus: false,
            };
            let tech = Tech {
                a: r.gen_range(0.0..1.0),
                b: r.gen_range(0.5..1.5),
                beta: r.gen_range(1.0..3.0),
            };
            let n: f64 = r.gen_range(1.0..20.0_f64).floor();
            let share = if r.gen::<bool>() {
                Share::Fraction(1.0 / n)
            } else {
                let own = r.gen_range(0.0..1.0);
                Share::Base {
                    own,
                    others: own * (n - 1.0) * r.gen_range(0.5..1.5),
                    n,
                }
            };
            let choice = Choice {
                tech,
                others: r.gen_range(0.0..(n - 1.0) * 0.8 + 0.01),
                share,
            };
            let (e, u) = best(&prefs, &choice, 0.0, 1.0, Search::Exact);
            let (g, ug) = best(&prefs, &choice, 0.0, 1.0, Search::Grid(4_000));
            assert!(
                u >= ug - 1e-6 * ug.abs().max(1.0),
                "case {case}: {prefs:?} {choice:?}: exact {e} {u} < grid {g} {ug}"
            );
        }
    }
}
