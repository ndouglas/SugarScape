//! The frame: opinion × time (DAWF's Figs. 3, 5–8), start against now
//! (DNAW Fig. 3, Weisbuch Figs. 4–5) and, on a torus, the lattice.

use crate::render::{lerp, Rgb};

/// Diagram columns: at most 240 kept periods and the current one.
pub const COLUMNS: usize = 241;
/// Periods the history keeps before it halves.
pub const KEPT: usize = COLUMNS - 1;
/// The diagram's and the panels' height; +1 at the top, −1 at the bottom.
pub const TALL: usize = 201;
/// Cells between panels.
pub const GAP: usize = 8;
/// Where the start-against-now panel starts; it is `TALL` square.
pub const SCATTER_X: usize = COLUMNS + GAP;
/// Where the torus starts, when there is one.
pub const TORUS_X: usize = SCATTER_X + TALL + GAP;

/// The gray of the scatter's diagonal (agents that never moved).
pub const DIAGONAL: Rgb = [0x3a, 0x38, 0x33];
/// Confident (small uncertainty) and uncertain, as in DAWF's figures.
pub const CONFIDENT: Rgb = [0xe0, 0x3c, 0x31];
pub const UNCERTAIN: Rgb = [0x4c, 0xc2, 0x5a];
/// The initial extremists under Role.
pub const PLUS: Rgb = [0xff, 0xc8, 0x3c];
pub const MINUS: Rgb = [0x4a, 0x9c, 0xff];
/// Moderates under Role, by opinion from −1 to +1.
pub const MODERATE_LOW: Rgb = [0x5c, 0x6a, 0x80];
pub const MODERATE_HIGH: Rgb = [0x80, 0x74, 0x5c];

/// The row of opinion `x`: +1 at the top, −1 at the bottom.
pub fn row(x: f64) -> usize {
    ((1.0 - x.clamp(-1.0, 1.0)) / 2.0 * (TALL - 1) as f64).round() as usize
}

/// The opinion at row `y`.
pub fn opinion_at(y: usize) -> f64 {
    1.0 - 2.0 * y as f64 / (TALL - 1) as f64
}

/// The scatter's column for a starting opinion: −1 at the left.
pub fn scatter_col(x: f64) -> usize {
    ((x.clamp(-1.0, 1.0) + 1.0) / 2.0 * (TALL - 1) as f64).round() as usize
}

/// Uncertainty `u` on a scale topping out at `top`.
pub fn uncertainty_color(u: f64, top: f64) -> Rgb {
    lerp(CONFIDENT, UNCERTAIN, (u / top).clamp(0.0, 1.0))
}

/// Opinion `x` on the Bounded Confidence model's spectrum (red at −1,
/// magenta at +1).
pub fn opinion_color(x: f64) -> Rgb {
    crate::opinions::hue((x.clamp(-1.0, 1.0) + 1.0) / 2.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_run_from_plus_one_at_the_top() {
        assert_eq!((row(1.0), row(-1.0), row(0.0)), (0, TALL - 1, 100));
        assert!((opinion_at(row(0.37)) - 0.37).abs() <= 1.0 / (TALL - 1) as f64);
        assert_eq!((scatter_col(-1.0), scatter_col(1.0)), (0, TALL - 1));
    }

    #[test]
    fn panels_sit_side_by_side() {
        assert_eq!((SCATTER_X, TORUS_X), (249, 458));
        assert_eq!(uncertainty_color(0.0, 1.0), CONFIDENT);
        assert_eq!(uncertainty_color(2.0, 1.0), UNCERTAIN);
        assert_eq!(opinion_color(-1.0), crate::opinions::hue(0.0));
    }
}
