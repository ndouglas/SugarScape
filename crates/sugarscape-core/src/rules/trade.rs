//! Agent trade rule T (Chapter IV) over pairs of goods. With each neighbor,
//! in random order: rank the pairs i < j by |ln MRS_A − ln MRS_B| (widest
//! first; ties to the lowest (i, j)) and make one exchange on the first pair
//! that passes the book's checks — the agent valuing good i more buys it with
//! good j at p = √(MRS_A·MRS_B), 1 of i for p of j if p ≥ 1, else 1/p of i
//! for 1 of j — as long as both agents' welfare over every good strictly
//! rises, their MRSs for the pair don't cross, and every holding stays
//! positive; then re-rank. Pairs with a non-finite, non-positive or equal
//! MRS are skipped. With two goods this is the book's loop.

use rand::seq::SliceRandom;

use crate::agent::AgentId;
use crate::config::MAX_GOODS;
use crate::econ::{mrs_n, welfare_n};
use crate::world::{Trade, World};

/// Safety bound on exchanges per pair per turn (welfare strictly rises each
/// time, so the loop ends long before this in practice).
const MAX_EXCHANGES: usize = 10_000;

pub(crate) fn act(world: &mut World, id: AgentId) {
    let pos = world.agent(id).expect("live agent").pos;
    let mut neighbors = world.torus.neighbors(pos);
    neighbors.shuffle(&mut world.rng);
    for q in neighbors {
        if let Some(other) = world.occupant(q) {
            trade_pair(world, id, other);
        }
    }
}

#[derive(Clone, Copy)]
struct Holdings {
    n: usize,
    w: [f64; MAX_GOODS],
    m: [f64; MAX_GOODS],
}

impl Holdings {
    fn of(world: &World, id: AgentId) -> Self {
        let n = world.config.goods.len();
        let fee = world.config.disease.active_fee();
        let a = world.agent(id).expect("live agent");
        Self {
            n,
            w: a.holdings,
            m: a.effective_metabolisms(n, fee),
        }
    }

    fn welfare(&self, w: &[f64; MAX_GOODS]) -> f64 {
        welfare_n(&w[..self.n], &self.m[..self.n])
    }

    fn mrs(&self, w: &[f64; MAX_GOODS], i: usize, j: usize) -> f64 {
        mrs_n(&w[..self.n], &self.m[..self.n], i, j)
    }
}

/// A pair of goods whose valuations differ: (i, j, MRS_A, MRS_B).
type Pair = (usize, usize, f64, f64);

/// Pairs i < j with finite, positive, unequal MRSs, widest
/// |ln MRS_A − ln MRS_B| first (a stable sort keeps (i, j) order on ties).
fn ranked_pairs(ha: &Holdings, hb: &Holdings) -> Vec<Pair> {
    let mut pairs: Vec<(f64, Pair)> = Vec::new();
    for i in 0..ha.n {
        for j in i + 1..ha.n {
            let (ma, mb) = (ha.mrs(&ha.w, i, j), hb.mrs(&hb.w, i, j));
            if ma.is_finite() && mb.is_finite() && ma > 0.0 && mb > 0.0 && ma != mb {
                pairs.push(((ma.ln() - mb.ln()).abs(), (i, j, ma, mb)));
            }
        }
    }
    pairs.sort_by(|x, y| y.0.total_cmp(&x.0));
    pairs.into_iter().map(|(_, pair)| pair).collect()
}

/// One exchange on `pair` if the book's checks pass; returns whether it
/// happened.
fn exchange(
    world: &mut World,
    (a, ha): (AgentId, &Holdings),
    (b, hb): (AgentId, &Holdings),
    pair: Pair,
) -> bool {
    let (i, j, ma, mb) = pair;
    let p = (ma * mb).sqrt();
    let ((buyer, hbuy), (seller, hsell)) = if ma > mb {
        ((a, ha), (b, hb))
    } else {
        ((b, hb), (a, ha))
    };
    let (qi, qj) = if p >= 1.0 { (1.0, p) } else { (1.0 / p, 1.0) };
    let mut buy = hbuy.w;
    buy[i] = hbuy.w[i] + qi;
    buy[j] = hbuy.w[j] - qj;
    let mut sell = hsell.w;
    sell[i] = hsell.w[i] - qi;
    sell[j] = hsell.w[j] + qj;
    let n = hbuy.n;
    let positive = buy[..n].iter().chain(&sell[..n]).all(|&x| x > 0.0);
    let better = hbuy.welfare(&buy) > hbuy.welfare(&hbuy.w)
        && hsell.welfare(&sell) > hsell.welfare(&hsell.w);
    let no_cross = hbuy.mrs(&buy, i, j) >= hsell.mrs(&sell, i, j);
    if !(positive && better && no_cross) {
        return false;
    }
    world.agent_mut(buyer).expect("buyer").holdings = buy;
    world.agent_mut(seller).expect("seller").holdings = sell;
    world.events.trades.push(Trade {
        buyer,
        seller,
        goods: (i, j),
        price: p,
        amount: qi,
    });
    true
}

pub(crate) fn trade_pair(world: &mut World, a: AgentId, b: AgentId) {
    for _ in 0..MAX_EXCHANGES {
        let (ha, hb) = (Holdings::of(world, a), Holdings::of(world, b));
        let pairs = ranked_pairs(&ha, &hb);
        if !pairs
            .into_iter()
            .any(|pair| exchange(world, (a, &ha), (b, &hb), pair))
        {
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::econ::{mrs, welfare, welfare_n};
    use crate::testkit::*;

    fn trader(w: &mut World, x: u32, sugar: f64, spice: f64) -> AgentId {
        let id = spawn(w, x, 0);
        let a = w.agent_mut(id).unwrap();
        a.holdings[0] = sugar;
        a.holdings[1] = spice;
        a.metabolism[0] = 1;
        a.metabolism[1] = 1;
        id
    }

    fn trader3(w: &mut World, x: u32, holdings: [f64; 3]) -> AgentId {
        let id = spawn(w, x, 0);
        let a = w.agent_mut(id).unwrap();
        a.holdings[..3].copy_from_slice(&holdings);
        a.metabolism[..3].copy_from_slice(&[1, 1, 1]);
        id
    }

    fn state(w: &World, id: AgentId) -> (f64, f64, f64, f64) {
        let a = w.agent(id).unwrap();
        let (m1, m2) = (f64::from(a.metabolism[0]), f64::from(a.metabolism[1]));
        (
            a.holdings[0],
            a.holdings[1],
            welfare(a.holdings[0], a.holdings[1], m1, m2),
            mrs(a.holdings[0], a.holdings[1], m1, m2),
        )
    }

    #[test]
    fn edgeworth_box_example_converges_without_crossing() {
        let mut w = blank_world(5, 5);
        add_goods(&mut w.config, 2);
        let a = trader(&mut w, 0, 5.0, 8.0);
        let b = trader(&mut w, 1, 15.0, 2.0);
        let (_, _, wa0, _) = state(&w, a);
        let (_, _, wb0, _) = state(&w, b);
        trade_pair(&mut w, a, b);
        let (sa, pa, wa, ma) = state(&w, a);
        let (sb, pb, wb, mb) = state(&w, b);
        assert!(wa > wa0 && wb > wb0, "both better off");
        assert!(ma >= mb, "MRSs did not cross (A valued sugar more)");
        assert!(
            (sa + sb - 20.0).abs() < 1e-9 && (pa + pb - 10.0).abs() < 1e-9,
            "goods conserved"
        );
        assert!(!w.events().trades.is_empty());
        let first = w.events().trades[0];
        assert_eq!((first.buyer, first.seller), (a, b), "A buys sugar");
        assert_eq!(first.goods, (0, 1));
        assert!((first.price - (1.6f64 * (2.0 / 15.0)).sqrt()).abs() < 1e-12);
    }

    #[test]
    fn equal_valuations_do_not_trade() {
        let mut w = blank_world(5, 5);
        add_goods(&mut w.config, 2);
        let a = trader(&mut w, 0, 10.0, 10.0);
        let b = trader(&mut w, 1, 20.0, 20.0);
        trade_pair(&mut w, a, b);
        assert!(w.events().trades.is_empty());
    }

    #[test]
    fn act_trades_with_neighbors_only() {
        let mut w = blank_world(10, 10);
        add_goods(&mut w.config, 2);
        let a = trader(&mut w, 0, 5.0, 8.0);
        trader(&mut w, 5, 15.0, 2.0); // not adjacent
        act(&mut w, a);
        assert!(w.events().trades.is_empty());
    }

    #[test]
    fn disease_fees_change_valuations() {
        // Both agents hold (100, 100) with metabolisms (1, 3): equal MRSs of
        // 1/3. Two diseases at fee 1 make A's metabolisms (3, 5), MRS 3/5, so
        // A now values sugar more and buys it.
        let mut w = blank_world(5, 5);
        add_goods(&mut w.config, 2);
        let a = trader(&mut w, 0, 100.0, 100.0);
        let b = trader(&mut w, 1, 100.0, 100.0);
        for id in [a, b] {
            w.agent_mut(id).unwrap().metabolism[1] = 3;
        }
        w.agent_mut(a).unwrap().diseases = vec![0, 1];
        trade_pair(&mut w, a, b);
        assert!(
            w.events().trades.is_empty(),
            "fees ignored while disease is off"
        );
        w.config.disease.enabled = true;
        trade_pair(&mut w, a, b);
        assert!(!w.events().trades.is_empty());
        assert_eq!(w.events().trades[0].buyer, a);
    }

    #[test]
    fn the_widest_valuation_gap_trades_first() {
        // MRS₀₁: A 1, B 2 (gap ln 2); MRS₀₂: A 4, B 0.5 (ln 8);
        // MRS₁₂: A 4, B 0.25 (ln 16) — so A first buys good 1 with good 2.
        let mut w = blank_world(5, 5);
        add_goods(&mut w.config, 3);
        let a = trader3(&mut w, 0, [10.0, 10.0, 40.0]);
        let b = trader3(&mut w, 1, [10.0, 20.0, 5.0]);
        trade_pair(&mut w, a, b);
        let first = w.events().trades[0];
        assert_eq!((first.buyer, first.seller, first.goods), (a, b, (1, 2)));
        assert!((first.price - (4.0f64 * 0.25).sqrt()).abs() < 1e-12);
        assert_eq!(first.amount, 1.0, "p = 1 ≥ 1: one unit of good 1");
    }

    #[test]
    fn equal_gaps_go_to_the_lowest_pair() {
        // MRS₀₁ and MRS₀₂ are both 2 (A) and 0.5 (B); MRS₁₂ is 1 for both.
        let mut w = blank_world(5, 5);
        add_goods(&mut w.config, 3);
        let a = trader3(&mut w, 0, [10.0, 20.0, 20.0]);
        let b = trader3(&mut w, 1, [20.0, 10.0, 10.0]);
        trade_pair(&mut w, a, b);
        assert_eq!(w.events().trades[0].goods, (0, 1));
        assert!(
            w.events().trades.iter().any(|t| t.goods == (0, 2)),
            "then re-ranks"
        );
    }

    #[test]
    fn trade_conserves_each_good_and_raises_both_welfares() {
        let mut w = blank_world(5, 5);
        add_goods(&mut w.config, 3);
        let a = trader3(&mut w, 0, [5.0, 30.0, 12.0]);
        let b = trader3(&mut w, 1, [25.0, 4.0, 9.0]);
        let welfare3 = |w: &World, id| {
            let x = w.agent(id).unwrap();
            welfare_n(&x.holdings[..3], &[1.0, 1.0, 1.0])
        };
        let (wa, wb) = (welfare3(&w, a), welfare3(&w, b));
        trade_pair(&mut w, a, b);
        assert!(!w.events().trades.is_empty());
        for (i, total) in [30.0, 34.0, 21.0].into_iter().enumerate() {
            let sum = w.agent(a).unwrap().holdings[i] + w.agent(b).unwrap().holdings[i];
            assert!((sum - total).abs() < 1e-9, "good {i}: {sum}");
        }
        assert!(welfare3(&w, a) > wa && welfare3(&w, b) > wb);
    }
}
