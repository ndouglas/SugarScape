//! Trade, credit and disease networks (Animations IV-4, IV-5 and Chapter V).

use std::collections::{BTreeMap, BTreeSet};

use crate::agent::AgentId;
use crate::geometry::Pos;
use crate::world::World;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CreditRole {
    None,
    Lender,
    Borrower,
    Both,
}

/// Every living agent's role in the outstanding loans.
pub fn credit_roles(world: &World) -> BTreeMap<AgentId, CreditRole> {
    let lenders: BTreeSet<AgentId> = world.loans().map(|l| l.lender).collect();
    let borrowers: BTreeSet<AgentId> = world.loans().map(|l| l.borrower).collect();
    world
        .agents()
        .map(|a| {
            let role = match (lenders.contains(&a.id), borrowers.contains(&a.id)) {
                (true, true) => CreditRole::Both,
                (true, false) => CreditRole::Lender,
                (false, true) => CreditRole::Borrower,
                (false, false) => CreditRole::None,
            };
            (a.id, role)
        })
        .collect()
}

fn edges(world: &World, pairs: impl Iterator<Item = (AgentId, AgentId)>) -> Vec<(Pos, Pos)> {
    let unique: BTreeSet<(AgentId, AgentId)> = pairs.map(|(a, b)| (a.min(b), a.max(b))).collect();
    unique
        .into_iter()
        .filter_map(|(a, b)| Some((world.agent(a)?.pos, world.agent(b)?.pos)))
        .collect()
}

/// Pairs of living agents who traded this tick.
pub fn trade_edges(world: &World) -> Vec<(Pos, Pos)> {
    edges(
        world,
        world.events().trades.iter().map(|t| (t.buyer, t.seller)),
    )
}

/// Lender–borrower pairs with an outstanding loan.
pub fn credit_edges(world: &World) -> Vec<(Pos, Pos)> {
    edges(world, world.loans().map(|l| (l.lender, l.borrower)))
}

/// Infector → infected pairs of living agents from this tick's transmissions
/// (Chapter V's disease network; outbreaks have no infector).
pub fn disease_edges(world: &World) -> Vec<(Pos, Pos)> {
    let pairs: BTreeSet<(AgentId, AgentId)> = world
        .events()
        .infections
        .iter()
        .filter_map(|i| Some((i.infector?, i.infected)))
        .collect();
    pairs
        .into_iter()
        .filter_map(|(a, b)| Some((world.agent(a)?.pos, world.agent(b)?.pos)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::*;
    use crate::world::Trade;

    #[test]
    fn edges_and_roles_follow_trades_and_loans() {
        let mut w = blank_world(5, 5);
        let a = spawn(&mut w, 0, 0);
        let b = spawn(&mut w, 1, 0);
        let c = spawn(&mut w, 2, 0);
        w.events.trades = vec![
            Trade {
                buyer: a,
                seller: b,
                goods: (0, 1),
                price: 1.0,
                amount: 1.0,
            },
            Trade {
                buyer: b,
                seller: a,
                goods: (0, 1),
                price: 1.0,
                amount: 1.0,
            },
        ];
        assert_eq!(
            trade_edges(&w),
            vec![(Pos::new(0, 0), Pos::new(1, 0))],
            "deduplicated"
        );
        w.originate_loan(a, b, 1.0);
        w.originate_loan(b, c, 1.0);
        let roles = credit_roles(&w);
        assert_eq!(roles[&a], CreditRole::Lender);
        assert_eq!(roles[&b], CreditRole::Both);
        assert_eq!(roles[&c], CreditRole::Borrower);
        assert_eq!(credit_edges(&w).len(), 2);
    }

    #[test]
    fn disease_edges_point_from_infector_to_infected() {
        use crate::world::Infection;
        let mut w = blank_world(5, 5);
        let a = spawn(&mut w, 0, 0);
        let b = spawn(&mut w, 1, 0);
        let infection = |infector, infected| Infection {
            infector,
            infected,
            disease: 0,
        };
        w.events.infections = vec![
            infection(Some(a), b),
            infection(Some(a), b),
            infection(None, a),
            infection(Some(b), 999),
        ];
        assert_eq!(
            disease_edges(&w),
            vec![(Pos::new(0, 0), Pos::new(1, 0))],
            "deduplicated; outbreaks and dead agents dropped"
        );
    }
}
