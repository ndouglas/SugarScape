//! Chapter IV's Cobb–Douglas welfare and the valuations derived from it.
//! Metabolisms are weights; with both zero the agent weighs goods equally.

use crate::config::MAX_GOODS;

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

/// Exponents mᵢ/m_T over n goods (1/n each when every metabolism is zero).
/// m_T is summed from 0.0 in good order, so for two goods it is exactly
/// m₁ + m₂ and the exponents equal `weights`'.
fn weights_n(m: &[f64]) -> [f64; MAX_GOODS] {
    let total = m.iter().fold(0.0, |sum, &x| sum + x);
    let mut out = [0.0; MAX_GOODS];
    for (slot, &x) in out.iter_mut().zip(m) {
        *slot = if total > 0.0 {
            x / total
        } else {
            1.0 / m.len() as f64
        };
    }
    out
}

/// Book eq. 1 over n goods: W = Π wᵢ^(mᵢ/m_T), negative holdings counting as
/// 0; the product starts at 1.0 and multiplies in good order.
pub fn welfare_n(w: &[f64], m: &[f64]) -> f64 {
    let a = weights_n(m);
    w.iter()
        .zip(a)
        .fold(1.0, |product, (&x, e)| product * x.max(0.0).powf(e))
}

/// Book eq. 6 over n goods: welfare as if `phi` periods of each metabolism
/// were already spent.
pub fn foresight_welfare_n(w: &[f64], m: &[f64], phi: u32) -> f64 {
    let phi = f64::from(phi);
    let mut spent = [0.0; MAX_GOODS];
    for ((slot, &x), &mi) in spent.iter_mut().zip(w).zip(m) {
        *slot = x - phi * mi;
    }
    welfare_n(&spent[..w.len()], m)
}

/// MRS for goods i < j: (wⱼ·mᵢ)/(mⱼ·wᵢ), units of j worth one unit of i
/// (book eq. 3 at (0, 1)); wⱼ/wᵢ when every metabolism is zero.
pub fn mrs_n(w: &[f64], m: &[f64], i: usize, j: usize) -> f64 {
    if m.iter().all(|&x| x == 0.0) {
        return w[j] / w[i];
    }
    (w[j] * m[i]) / (m[j] * w[i])
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

    /// Holdings (some ≤ 0), integer metabolisms 0–6 plus a disease fee of 0,
    /// 1 or 2.5 per disease, and φ 0–11: what the simulation produces.
    fn samples() -> Vec<([f64; 2], [f64; 2], u32)> {
        use rand::Rng;
        let mut rng = crate::rng::seeded(11);
        (0..20_000)
            .map(|_| {
                let w = [rng.gen_range(-5.0..80.0), rng.gen_range(-5.0..80.0)];
                let fee = [0.0, 1.0, 2.5][rng.gen_range(0..3)];
                let k = f64::from(rng.gen_range(0..4u32));
                let m = [
                    f64::from(rng.gen_range(0..7u32)) + fee * k,
                    f64::from(rng.gen_range(0..7u32)) + fee * k,
                ];
                (w, m, rng.gen_range(0..12))
            })
            .collect()
    }

    fn same(a: f64, b: f64) -> bool {
        a.to_bits() == b.to_bits() || (a.is_nan() && b.is_nan())
    }

    #[test]
    fn n_good_forms_equal_the_two_good_forms_bit_for_bit() {
        for (w, m, phi) in samples() {
            assert!(
                same(welfare_n(&w, &m), welfare(w[0], w[1], m[0], m[1])),
                "{w:?} {m:?}"
            );
            assert!(
                same(
                    foresight_welfare_n(&w, &m, phi),
                    foresight_welfare(w[0], w[1], m[0], m[1], phi)
                ),
                "{w:?} {m:?} {phi}"
            );
            assert!(
                same(mrs_n(&w, &m, 0, 1), mrs(w[0], w[1], m[0], m[1])),
                "{w:?} {m:?}"
            );
        }
    }

    #[test]
    fn one_good_welfare_is_the_two_good_form_with_a_weightless_second_good() {
        for (w, m, phi) in samples() {
            if m[0] == 0.0 {
                continue;
            }
            assert!(same(
                welfare_n(&w[..1], &m[..1]),
                welfare(w[0], w[1], m[0], 0.0)
            ));
            assert!(same(
                foresight_welfare_n(&w[..1], &m[..1], phi),
                foresight_welfare(w[0], w[1], m[0], 0.0, phi)
            ));
        }
    }

    #[test]
    fn three_good_welfare_and_mrs() {
        // Equal weights: (4·9·16)^(1/3) = 576^(1/3).
        assert!(close(
            welfare_n(&[4.0, 9.0, 16.0], &[1.0, 1.0, 1.0]),
            576f64.powf(1.0 / 3.0)
        ));
        assert!(close(
            welfare_n(&[4.0, 9.0, 16.0], &[0.0, 0.0, 0.0]),
            576f64.powf(1.0 / 3.0)
        ));
        assert_eq!(welfare_n(&[4.0, 0.0, 16.0], &[1.0, 1.0, 1.0]), 0.0);
        // Units of good 2 worth one unit of good 1: (16·2)/(1·9).
        assert!(close(
            mrs_n(&[4.0, 9.0, 16.0], &[1.0, 2.0, 1.0], 1, 2),
            32.0 / 9.0
        ));
        assert!(close(mrs_n(&[4.0, 9.0, 16.0], &[0.0, 0.0, 0.0], 0, 2), 4.0));
        assert!(
            mrs_n(&[4.0, 9.0, 16.0], &[1.0, 0.0, 0.0], 1, 2).is_nan(),
            "two weightless goods have no exchange rate"
        );
        assert!(close(
            foresight_welfare_n(&[10.0, 10.0, 10.0], &[1.0, 1.0, 2.0], 2),
            welfare_n(&[8.0, 8.0, 6.0], &[1.0, 1.0, 2.0])
        ));
    }
}
