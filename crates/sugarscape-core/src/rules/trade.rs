//! Agent trade rule T (Chapter IV). With each neighbor, in random order:
//! while their MRSs differ, the higher-MRS agent buys sugar with spice at
//! p = √(MRS_A·MRS_B) — 1 sugar for p spice if p ≥ 1, else 1/p sugar for 1
//! spice — as long as both agents' welfare strictly rises, their MRSs don't
//! cross, and both keep positive holdings. Each exchange is one trade.

use rand::seq::SliceRandom;

use crate::agent::AgentId;
use crate::econ::{mrs, welfare};
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

struct Holdings {
    sugar: f64,
    spice: f64,
    m1: f64,
    m2: f64,
}

impl Holdings {
    fn of(world: &World, id: AgentId) -> Self {
        let fee = world.config.disease.active_fee();
        let a = world.agent(id).expect("live agent");
        Self {
            sugar: a.holdings[0],
            spice: a.holdings[1],
            m1: a.effective_metabolism(0, fee),
            m2: a.effective_metabolism(1, fee),
        }
    }
    fn welfare(&self, sugar: f64, spice: f64) -> f64 {
        welfare(sugar, spice, self.m1, self.m2)
    }
    fn mrs(&self, sugar: f64, spice: f64) -> f64 {
        mrs(sugar, spice, self.m1, self.m2)
    }
}

pub(crate) fn trade_pair(world: &mut World, a: AgentId, b: AgentId) {
    for _ in 0..MAX_EXCHANGES {
        let (ha, hb) = (Holdings::of(world, a), Holdings::of(world, b));
        let (ma, mb) = (ha.mrs(ha.sugar, ha.spice), hb.mrs(hb.sugar, hb.spice));
        if !(ma.is_finite() && mb.is_finite() && ma > 0.0 && mb > 0.0) || ma == mb {
            return;
        }
        let p = (ma * mb).sqrt();
        let ((buyer, hbuy), (seller, hsell)) = if ma > mb {
            ((a, ha), (b, hb))
        } else {
            ((b, hb), (a, ha))
        };
        let (sugar, spice) = if p >= 1.0 { (1.0, p) } else { (1.0 / p, 1.0) };
        let buy = (hbuy.sugar + sugar, hbuy.spice - spice);
        let sell = (hsell.sugar - sugar, hsell.spice + spice);
        let positive = buy.0 > 0.0 && buy.1 > 0.0 && sell.0 > 0.0 && sell.1 > 0.0;
        let better = hbuy.welfare(buy.0, buy.1) > hbuy.welfare(hbuy.sugar, hbuy.spice)
            && hsell.welfare(sell.0, sell.1) > hsell.welfare(hsell.sugar, hsell.spice);
        let no_cross = hbuy.mrs(buy.0, buy.1) >= hsell.mrs(sell.0, sell.1);
        if !(positive && better && no_cross) {
            return;
        }
        let x = world.agent_mut(buyer).expect("buyer");
        x.holdings[0] = buy.0;
        x.holdings[1] = buy.1;
        let y = world.agent_mut(seller).expect("seller");
        y.holdings[0] = sell.0;
        y.holdings[1] = sell.1;
        world.events.trades.push(Trade {
            buyer,
            seller,
            price: p,
            sugar,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        w.config.spice.enabled = true;
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
        assert!((first.price - (1.6f64 * (2.0 / 15.0)).sqrt()).abs() < 1e-12);
    }

    #[test]
    fn equal_valuations_do_not_trade() {
        let mut w = blank_world(5, 5);
        w.config.spice.enabled = true;
        let a = trader(&mut w, 0, 10.0, 10.0);
        let b = trader(&mut w, 1, 20.0, 20.0);
        trade_pair(&mut w, a, b);
        assert!(w.events().trades.is_empty());
    }

    #[test]
    fn act_trades_with_neighbors_only() {
        let mut w = blank_world(10, 10);
        w.config.spice.enabled = true;
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
        w.config.spice.enabled = true;
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
}
