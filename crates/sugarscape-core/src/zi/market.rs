//! The markets' schedules — Gode and Sunder's five, read from their figures
//! (the text's anchors: P₀ 69 and 170 in markets 2 and 4, volumes 24 and 6
//! in markets 1 and 3, and Table 2's ZI-U efficiencies, which follow from the
//! schedules alone), and Cliff's five, read from his figures — and their
//! competitive equilibrium.

use super::config::{Market, Shift, ZiConfig, SHIFT};

/// Six traders holding the same units.
fn six(units: &[u32]) -> Vec<Vec<u32>> {
    vec![units.to_vec(); 6]
}

/// One unit each at these limits.
fn one_each(limits: impl IntoIterator<Item = u32>) -> Vec<Vec<u32>> {
    limits.into_iter().map(|v| vec![v]).collect()
}

/// Market 5's staircase: 18 intramarginal units a side, evenly from `from`
/// to `to`, dealt in turn to three traders; three more traders hold seven
/// extramarginal units each at `beyond`.
fn staircase(from: u32, to: u32, beyond: u32) -> Vec<Vec<u32>> {
    let steps: Vec<u32> = (0..18u32)
        .map(|i| {
            let (a, b) = (f64::from(from), f64::from(to));
            (a + (b - a) * f64::from(i) / 17.0).round() as u32
        })
        .collect();
    let mut out: Vec<Vec<u32>> = (0..3)
        .map(|k| steps.iter().skip(k).step_by(3).copied().collect())
        .collect();
    out.extend(vec![vec![beyond; 7]; 3]);
    out
}

/// Each buyer's values and each seller's costs, in trading order, before any shift.
pub fn schedules(c: &ZiConfig) -> (Vec<Vec<u32>>, Vec<Vec<u32>>) {
    match c.market {
        Market::Gs1 => (
            six(&[102, 97, 92, 87, 82, 77]),
            six(&[34, 46, 58, 70, 82, 94]),
        ),
        Market::Gs2 => (
            six(&[117, 105, 93, 81, 69, 57]),
            six(&[49, 54, 59, 64, 69, 74]),
        ),
        Market::Gs3 => (six(&[133, 95, 90]), six(&[90, 95, 100])),
        Market::Gs4 => (
            six(&[180, 175, 170, 165, 160]),
            six(&[90, 142, 170, 190, 198]),
        ),
        Market::Gs5 => (staircase(159, 131, 127), staircase(96, 124, 131)),
        Market::Symmetric => (
            one_each((0..11).map(|i| 75 + 25 * i)),
            one_each((0..11).map(|i| 75 + 25 * i)),
        ),
        Market::FlatSupply => (one_each((0..11).map(|i| 75 + 25 * i)), one_each([200; 11])),
        Market::ExcessDemand => (one_each([200; 11]), one_each([50; 6])),
        Market::ExcessSupply => (one_each([320; 6]), one_each([200; 11])),
        Market::Retail => (
            one_each((0..12).map(|i| 100 + 25 * i)),
            one_each((0..11).map(|i| 75 + 25 * i)),
        ),
        Market::Custom => (c.buyers.clone(), c.sellers.clone()),
    }
}

/// The schedules in force in `period` (1-based): shifted from `shift_at`.
pub fn schedules_at(c: &ZiConfig, period: u64) -> (Vec<Vec<u32>>, Vec<Vec<u32>>) {
    let (mut b, mut s) = schedules(c);
    if period >= u64::from(c.shift_at) {
        match c.shift {
            Shift::None => {}
            Shift::Demand => b.iter_mut().flatten().for_each(|v| *v += SHIFT),
            Shift::Supply => s.iter_mut().flatten().for_each(|v| *v -= SHIFT),
        }
    }
    (b, s)
}

/// The competitive equilibrium of a market.
#[derive(Clone, Debug, PartialEq)]
pub struct Equilibrium {
    /// Units that can trade without loss (value ≥ cost), Cliff's Q₀; Gode
    /// and Sunder's volumes leave out the units whose value equals their cost.
    pub quantity: u32,
    /// The price: the middle of the range that clears `quantity`.
    pub price: f64,
    /// The largest total profit (consumer plus producer surplus).
    pub surplus: i64,
}

pub fn equilibrium(buyers: &[Vec<u32>], sellers: &[Vec<u32>]) -> Equilibrium {
    let mut d: Vec<u32> = buyers.iter().flatten().copied().collect();
    let mut c: Vec<u32> = sellers.iter().flatten().copied().collect();
    d.sort_unstable_by(|a, b| b.cmp(a));
    c.sort_unstable();
    let mut q = 0;
    while q < d.len().min(c.len()) && d[q] >= c[q] {
        q += 1;
    }
    let surplus = (0..q).map(|i| i64::from(d[i]) - i64::from(c[i])).sum();
    let price = if q == 0 {
        (f64::from(d.first().copied().unwrap_or(0)) + f64::from(c.first().copied().unwrap_or(0)))
            / 2.0
    } else {
        let lo = c[q - 1].max(d.get(q).copied().unwrap_or(0));
        let hi = d[q - 1].min(c.get(q).copied().unwrap_or(u32::MAX));
        (f64::from(lo) + f64::from(hi)) / 2.0
    };
    Equilibrium {
        quantity: q as u32,
        price,
        surplus,
    }
}

/// Each trader's profit at the equilibrium price (buyers first).
pub fn equilibrium_profits(buyers: &[Vec<u32>], sellers: &[Vec<u32>], price: f64) -> Vec<f64> {
    buyers
        .iter()
        .map(|t| t.iter().map(|&v| (f64::from(v) - price).max(0.0)).sum())
        .chain(
            sellers
                .iter()
                .map(|t| t.iter().map(|&c| (price - f64::from(c)).max(0.0)).sum()),
        )
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn market(m: Market) -> (Vec<Vec<u32>>, Vec<Vec<u32>>) {
        schedules(&ZiConfig {
            market: m,
            ..ZiConfig::default()
        })
    }

    /// ZI-U trades every unit: its efficiency is the whole schedules' surplus
    /// over the equilibrium's.
    fn all_units(b: &[Vec<u32>], s: &[Vec<u32>]) -> f64 {
        let total: i64 = b.iter().flatten().map(|&v| i64::from(v)).sum::<i64>()
            - s.iter().flatten().map(|&v| i64::from(v)).sum::<i64>();
        100.0 * total as f64 / equilibrium(b, s).surplus as f64
    }

    #[test]
    fn gode_and_sunders_markets_meet_the_texts_anchors() {
        let (b, s) = market(Market::Gs2);
        assert_eq!(equilibrium(&b, &s).price, 69.0);
        let (b, s) = market(Market::Gs4);
        assert_eq!(equilibrium(&b, &s).price, 170.0);
        // Volumes 24 and 6 leave out the zero-surplus units (value = cost).
        let (b, s) = market(Market::Gs1);
        assert_eq!(equilibrium(&b, &s).quantity, 30);
        let (b, s) = market(Market::Gs3);
        assert_eq!(equilibrium(&b, &s).quantity, 12);
        // Table 2's ZI-U efficiencies.
        for (m, want) in [
            (Market::Gs1, 90.0),
            (Market::Gs2, 90.0),
            (Market::Gs3, 76.7),
            (Market::Gs4, 48.8),
        ] {
            let (b, s) = market(m);
            assert!(
                (all_units(&b, &s) - want).abs() < 0.05,
                "{m:?}: {}",
                all_units(&b, &s)
            );
        }
        let (b, s) = market(Market::Gs5);
        assert_eq!((b.len(), s.len()), (6, 6));
        assert_eq!(equilibrium(&b, &s).price, 129.0);
        assert!((all_units(&b, &s) - 86.0).abs() < 1.0);
    }

    #[test]
    fn cliffs_markets_clear_at_200_and_retail_at_225() {
        for m in [
            Market::Symmetric,
            Market::FlatSupply,
            Market::ExcessDemand,
            Market::ExcessSupply,
        ] {
            let (b, s) = market(m);
            let e = equilibrium(&b, &s);
            assert_eq!((e.price, e.quantity), (200.0, 6), "{m:?}");
        }
        let (b, s) = market(Market::Retail);
        assert_eq!((b.len(), s.len()), (12, 11));
        let e = equilibrium(&b, &s);
        assert_eq!((e.price, e.quantity), (225.0, 7));
    }

    #[test]
    fn shifts_move_one_side_by_50() {
        let c = ZiConfig {
            market: Market::Symmetric,
            price_max: 400,
            shift: Shift::Demand,
            ..ZiConfig::default()
        };
        let (b, s) = schedules_at(&c, 11);
        assert_eq!(equilibrium(&b, &s).price, 225.0);
        let (b, s) = schedules_at(&c, 10);
        assert_eq!(equilibrium(&b, &s).price, 200.0);
        let c = ZiConfig {
            shift: Shift::Supply,
            ..c
        };
        let (b, s) = schedules_at(&c, 11);
        assert_eq!(equilibrium(&b, &s).price, 175.0);
    }

    #[test]
    fn equilibrium_profits_sum_to_the_surplus() {
        let (b, s) = market(Market::Gs1);
        let e = equilibrium(&b, &s);
        let p = equilibrium_profits(&b, &s, e.price);
        assert_eq!(p.iter().sum::<f64>() as i64, e.surplus);
    }
}
