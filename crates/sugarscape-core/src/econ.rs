//! Chapter IV's Cobb–Douglas welfare and the valuations derived from it.
//! Metabolisms are weights; with both zero the agent weighs goods equally.

/// Exponents m₁/m_T and m₂/m_T (½, ½ when both metabolisms are zero).
fn weights(m1: f64, m2: f64) -> (f64, f64) {
    let mt = m1 + m2;
    if mt > 0.0 {
        (m1 / mt, m2 / mt)
    } else {
        (0.5, 0.5)
    }
}

/// Book eq. 1: W = w₁^(m₁/m_T) · w₂^(m₂/m_T); negative holdings count as 0.
pub fn welfare(w1: f64, w2: f64, m1: f64, m2: f64) -> f64 {
    let (a, b) = weights(m1, m2);
    w1.max(0.0).powf(a) * w2.max(0.0).powf(b)
}

/// Book eq. 6: welfare as if `phi` periods of metabolism were already spent.
pub fn foresight_welfare(w1: f64, w2: f64, m1: f64, m2: f64, phi: u32) -> f64 {
    let phi = f64::from(phi);
    welfare(w1 - phi * m1, w2 - phi * m2, m1, m2)
}

/// Book eq. 3: MRS = (w₂/m₂)/(w₁/m₁), the spice value of one unit of sugar.
pub fn mrs(w1: f64, w2: f64, m1: f64, m2: f64) -> f64 {
    if m1 == 0.0 && m2 == 0.0 {
        return w2 / w1;
    }
    (w2 * m1) / (m2 * w1)
}

/// Sugar an agent would hold at price `p` (spice per sugar) if it could
/// re-trade its whole bundle: the Cobb–Douglas share of its wealth.
pub fn sugar_demand(p: f64, w1: f64, w2: f64, m1: f64, m2: f64) -> f64 {
    let (a, _) = weights(m1, m2);
    a * (p * w1 + w2) / p
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-12
    }

    #[test]
    fn welfare_is_cobb_douglas_in_metabolism_weights() {
        assert!(close(welfare(4.0, 9.0, 1.0, 1.0), 6.0));
        assert!(close(welfare(8.0, 1.0, 3.0, 0.0), 8.0));
        assert!(close(welfare(4.0, 9.0, 0.0, 0.0), 6.0));
        assert_eq!(welfare(0.0, 9.0, 1.0, 1.0), 0.0);
    }

    #[test]
    fn foresight_subtracts_future_metabolism_and_floors_at_zero() {
        assert!(close(
            foresight_welfare(10.0, 10.0, 1.0, 1.0, 2),
            welfare(8.0, 8.0, 1.0, 1.0)
        ));
        assert_eq!(foresight_welfare(5.0, 50.0, 1.0, 1.0, 10), 0.0);
    }

    #[test]
    fn mrs_is_spice_per_sugar() {
        assert!(close(mrs(10.0, 20.0, 1.0, 1.0), 2.0));
        assert!(close(mrs(10.0, 20.0, 2.0, 1.0), 4.0));
        assert_eq!(mrs(10.0, 20.0, 0.0, 1.0), 0.0);
        assert!(mrs(10.0, 20.0, 1.0, 0.0).is_infinite());
    }

    #[test]
    fn sugar_demand_is_the_cobb_douglas_share_of_wealth() {
        // Wealth at p = 1 is 30 spice-units; half of it is spent on sugar.
        assert!(close(sugar_demand(1.0, 10.0, 20.0, 1.0, 1.0), 15.0));
        assert!(close(sugar_demand(2.0, 10.0, 20.0, 1.0, 1.0), 10.0));
    }
}
