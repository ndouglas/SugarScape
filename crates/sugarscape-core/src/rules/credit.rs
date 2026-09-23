//! Agent credit rule L_{d,r} (Chapter IV) in the n-commodity form of the
//! book's footnote 55: each good is lent and borrowed separately. Lenders: agents too old to have children lend up to half their
//! sugar; fertile agents lend sugar above their birth endowment. Borrowers:
//! fertile agents short of their endowment with positive income this turn.
//! A loan of P is creditworthy when income × d ≥ P × (1 + r/100 × d)
//! (interpretation of the book's "credit-worthy for a loan written at terms
//! specified by the lender"). At the due tick the borrower pays in full, or
//! pays half its sugar and the remainder is re-lent on the loan's own terms
//! (a default). Each loan carries the duration and rate it was written on. A dead borrower's loans are the lender's loss; a dead lender's
//! loans are cancelled unless inheritance (I) is on, when its living children
//! split the claim (a child who is the borrower has its share forgiven).

use rand::seq::SliceRandom;

use crate::agent::{Agent, AgentId};
use crate::config::MAX_GOODS;
use crate::rules::Harvest;
use crate::world::World;

fn fertile_age(a: &Agent) -> bool {
    (a.fertility_onset..=a.fertility_end).contains(&a.age)
}

/// How much of `good` agent `a` may lend now.
pub(crate) fn lendable(a: &Agent, good: usize) -> f64 {
    let (have, born_with) = (a.holdings[good], a.initial[good]);
    if a.age > a.fertility_end {
        (have / 2.0).max(0.0)
    } else if fertile_age(a) && have > born_with {
        have - born_with
    } else {
        0.0
    }
}

/// Σ(amount due / the loan's own duration) over `id`'s loans of `good`.
fn obligations(world: &World, id: AgentId, good: usize) -> f64 {
    world
        .loans()
        .filter(|l| l.borrower == id && l.good == good)
        .map(|l| l.due / f64::from(l.duration))
        .sum()
}

/// Sets each good's income = gathered − effective metabolism − per-tick
/// obligations in that good.
pub(crate) fn record_income(world: &mut World, id: AgentId, harvest: &Harvest) {
    let n = world.config.goods.len();
    let fee = world.config.disease.active_fee();
    let owed: [f64; MAX_GOODS] = std::array::from_fn(|i| {
        if i < n {
            obligations(world, id, i)
        } else {
            0.0
        }
    });
    let a = world.agent_mut(id).expect("live agent");
    let income: [f64; MAX_GOODS] = std::array::from_fn(|i| {
        if i < n {
            harvest.gathered[i] - a.effective_metabolism(i, fee) - owed[i]
        } else {
            0.0
        }
    });
    a.income = income;
}

/// A fertile-aged agent short of its birth endowment of some goods, with
/// positive income in them, asks its neighbors (shuffled once) for each such
/// good in good order.
pub(crate) fn borrow(world: &mut World, id: AgentId) {
    let c = world.config.credit;
    let n = world.config.goods.len();
    let me = world.agent(id).expect("live agent");
    let short: Vec<usize> = (0..n)
        .filter(|&i| fertile_age(me) && me.holdings[i] < me.initial[i] && me.income[i] > 0.0)
        .collect();
    if short.is_empty() {
        return;
    }
    let d = f64::from(c.duration);
    let mut neighbors = world.torus.neighbors(me.pos);
    neighbors.shuffle(&mut world.rng);
    for good in short {
        let me = world.agent(id).expect("live agent");
        let mut need = me.initial[good] - me.holdings[good];
        let mut capacity = me.income[good] * d / (1.0 + c.rate / 100.0 * d);
        for &q in &neighbors {
            if need <= 0.0 || capacity <= 0.0 {
                break;
            }
            let Some(lender) = world.occupant(q) else {
                continue;
            };
            let amount = lendable(world.agent(lender).expect("occupant"), good)
                .min(need)
                .min(capacity);
            if amount <= 0.0 {
                continue;
            }
            world.agent_mut(lender).expect("lender").holdings[good] -= amount;
            world.agent_mut(id).expect("borrower").holdings[good] += amount;
            world.originate_loan(lender, id, good, amount);
            world.events.loans_made += 1;
            world.events.amount_lent += amount;
            need -= amount;
            capacity -= amount;
        }
    }
}

/// Settles loans due this tick, oldest first, each in its own good.
pub(crate) fn settle(world: &mut World) {
    let due: Vec<_> = world
        .loans()
        .filter(|l| l.due_tick == world.tick)
        .map(|l| l.id)
        .collect();
    for loan_id in due {
        let loan = world.remove_loan(loan_id).expect("due loan");
        let available = world
            .agent(loan.borrower)
            .expect("borrowers' loans die with them")
            .holdings[loan.good];
        // Paying in full must leave the borrower alive (holding > 0);
        // otherwise it pays half and the rest rolls over.
        let paid = if available > loan.due {
            loan.due
        } else {
            available / 2.0
        };
        world.agent_mut(loan.borrower).expect("borrower").holdings[loan.good] -= paid;
        world
            .agent_mut(loan.lender)
            .expect("lenders' loans die with them")
            .holdings[loan.good] += paid;
        if paid < loan.due {
            world.originate_loan_on(
                loan.lender,
                loan.borrower,
                loan.good,
                loan.due - paid,
                loan.duration,
                loan.rate,
            );
            world.events.defaults += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::Harvest;
    use crate::testkit::*;
    use crate::world::DeathCause;

    fn credit_world() -> World {
        let mut w = blank_world(5, 5);
        w.config.sex.enabled = true;
        w.config.credit.enabled = true;
        w.config.credit.duration = 10;
        w.config.credit.rate = 10.0;
        w
    }

    /// A fertile borrower short of its endowment (sugar 4 of 10) with income 5.
    fn borrower(w: &mut World) -> AgentId {
        let id = spawn(w, 2, 2);
        let a = w.agent_mut(id).unwrap();
        a.holdings[0] = 4.0;
        a.income[0] = 5.0;
        id
    }

    /// An old lender (past fertility) holding 40 sugar.
    fn lender(w: &mut World) -> AgentId {
        let id = spawn(w, 2, 1);
        let a = w.agent_mut(id).unwrap();
        a.age = 70;
        a.holdings[0] = 40.0;
        id
    }

    #[test]
    fn lendable_amounts_follow_the_book() {
        let mut w = credit_world();
        let old = lender(&mut w);
        assert_eq!(lendable(w.agent(old).unwrap(), 0), 20.0);
        let young = spawn(&mut w, 0, 0); // fertile, sugar 10 = endowment 10
        w.agent_mut(young).unwrap().holdings[0] = 16.0;
        assert_eq!(lendable(w.agent(young).unwrap(), 0), 6.0);
        w.agent_mut(young).unwrap().age = 5;
        assert_eq!(
            lendable(w.agent(young).unwrap(), 0),
            0.0,
            "too young to lend"
        );
    }

    #[test]
    fn borrowing_fills_the_need_from_neighbors() {
        let mut w = credit_world();
        let b = borrower(&mut w);
        let l = lender(&mut w);
        borrow(&mut w, b);
        assert_eq!(
            w.agent(b).unwrap().holdings[0],
            10.0,
            "borrowed its 6 shortfall"
        );
        assert_eq!(w.agent(l).unwrap().holdings[0], 34.0);
        let loan = *w.loans().next().unwrap();
        assert_eq!((loan.lender, loan.borrower, loan.principal), (l, b, 6.0));
        assert!((loan.due - 12.0).abs() < 1e-12, "6 × (1 + 0.1 × 10)");
        assert_eq!(loan.due_tick, w.tick + 10);
        assert_eq!((w.events().loans_made, w.events().amount_lent), (1, 6.0));
    }

    #[test]
    fn creditworthiness_caps_the_principal() {
        let mut w = credit_world();
        let b = borrower(&mut w);
        w.agent_mut(b).unwrap().income[0] = 0.6; // 0.6 × 10 / 2 = 3 at most
        lender(&mut w);
        borrow(&mut w, b);
        assert!((w.agent(b).unwrap().holdings[0] - 7.0).abs() < 1e-12);
    }

    #[test]
    fn settlement_repays_or_rolls_over() {
        let mut w = credit_world();
        let b = borrower(&mut w);
        let l = lender(&mut w);
        borrow(&mut w, b);
        w.tick += 10;
        w.agent_mut(b).unwrap().holdings[0] = 20.0;
        settle(&mut w);
        assert_eq!(w.agent(b).unwrap().holdings[0], 8.0);
        assert_eq!(w.agent(l).unwrap().holdings[0], 46.0);
        assert_eq!(w.loans().count(), 0);

        let b2 = spawn(&mut w, 0, 0);
        w.agent_mut(b2).unwrap().holdings[0] = 4.0;
        let id = w.originate_loan(l, b2, 0, 6.0);
        let due_tick = w.loans().find(|x| x.id == id).unwrap().due_tick;
        w.tick = due_tick;
        settle(&mut w);
        assert_eq!(w.agent(b2).unwrap().holdings[0], 2.0, "paid half");
        let rolled = w.loans().next().unwrap();
        assert!((rolled.principal - 10.0).abs() < 1e-12, "12 due − 2 paid");
        assert_eq!(rolled.due_tick, due_tick + 10);
        assert_eq!(w.events().defaults, 1);
    }

    #[test]
    fn paying_exactly_the_amount_due_would_starve_so_it_rolls_over() {
        let mut w = credit_world();
        let b = borrower(&mut w);
        let l = lender(&mut w);
        borrow(&mut w, b);
        let loan = *w.loans().next().unwrap();
        w.tick = loan.due_tick;
        w.agent_mut(b).unwrap().holdings[0] = loan.due;
        settle(&mut w);
        assert_eq!(w.agent(b).unwrap().holdings[0], loan.due / 2.0, "paid half");
        let rolled = w.loans().next().expect("the rest rolls over");
        assert!((rolled.principal - loan.due / 2.0).abs() < 1e-12);
        assert_eq!((rolled.lender, rolled.borrower), (l, b));
        assert_eq!(w.events().defaults, 1);
    }

    #[test]
    fn loans_keep_the_terms_they_were_written_on() {
        let mut w = credit_world();
        let b = borrower(&mut w);
        let l = lender(&mut w);
        borrow(&mut w, b); // 6 at d = 10, r = 10 → 12 due
        let loan = *w.loans().next().unwrap();
        assert_eq!((loan.duration, loan.rate), (10, 10.0));
        w.config.credit.duration = 5;
        w.config.credit.rate = 0.0;
        w.agent_mut(b).unwrap().metabolism[0] = 0;
        record_income(&mut w, b, &Harvest::default());
        assert!(
            (w.agent(b).unwrap().income[0] + 1.2).abs() < 1e-12,
            "12 due over the loan's own 10 ticks"
        );
        w.tick = loan.due_tick;
        w.agent_mut(b).unwrap().holdings[0] = 4.0;
        settle(&mut w);
        let rolled = *w.loans().next().unwrap();
        assert_eq!(rolled.lender, l);
        assert!((rolled.principal - 10.0).abs() < 1e-12, "12 due − 2 paid");
        assert_eq!((rolled.duration, rolled.rate), (10, 10.0));
        assert!((rolled.due - 20.0).abs() < 1e-12, "10 × (1 + 0.1 × 10)");
        assert_eq!(rolled.due_tick, loan.due_tick + 10);
    }

    #[test]
    fn deaths_resolve_loans() {
        let mut w = credit_world();
        let b = borrower(&mut w);
        let l = lender(&mut w);
        borrow(&mut w, b);
        w.kill(b, DeathCause::Starvation);
        assert_eq!(w.loans().count(), 0, "lender takes the loss");

        let b = borrower(&mut w);
        borrow(&mut w, b);
        let child = spawn(&mut w, 4, 4);
        w.agent_mut(l).unwrap().children = vec![child];
        w.config.inheritance.enabled = true;
        w.kill(l, DeathCause::OldAge);
        let loan = w.loans().next().expect("claim passes to the child");
        assert_eq!((loan.lender, loan.borrower), (child, b));
    }

    #[test]
    fn a_borrower_does_not_inherit_a_claim_on_itself() {
        let mut w = credit_world();
        w.config.inheritance.enabled = true;
        let b = borrower(&mut w);
        let l = lender(&mut w);
        borrow(&mut w, b);
        w.agent_mut(l).unwrap().children = vec![b];
        w.kill(l, DeathCause::OldAge);
        assert_eq!(w.loans().count(), 0, "the only heir is the borrower");

        let mut w = credit_world();
        w.config.inheritance.enabled = true;
        let b = borrower(&mut w);
        let l = lender(&mut w);
        borrow(&mut w, b);
        let due = w.loans().next().unwrap().due;
        let sibling = spawn(&mut w, 4, 4);
        w.agent_mut(l).unwrap().children = vec![b, sibling];
        w.kill(l, DeathCause::OldAge);
        let loans: Vec<_> = w.loans().copied().collect();
        assert_eq!(loans.len(), 1);
        assert_eq!((loans[0].lender, loans[0].borrower), (sibling, b));
        assert!(
            (loans[0].due - due / 2.0).abs() < 1e-12,
            "the sibling's share"
        );
    }

    #[test]
    fn income_is_gathered_minus_metabolism_minus_obligations() {
        let mut w = credit_world();
        let b = borrower(&mut w);
        let l = lender(&mut w);
        w.agent_mut(b).unwrap().metabolism[0] = 2;
        w.originate_loan(l, b, 0, 5.0); // due 10 over 10 ticks → 1 per tick
        record_income(&mut w, b, &Harvest::of(&[6.0]));
        assert!((w.agent(b).unwrap().income[0] - 3.0).abs() < 1e-12);
    }

    #[test]
    fn income_counts_the_disease_fee() {
        let mut w = credit_world();
        w.config.disease.enabled = true;
        w.config.disease.fee = 2.0;
        let b = borrower(&mut w);
        {
            let a = w.agent_mut(b).unwrap();
            a.metabolism[0] = 1;
            a.diseases = vec![0];
        }
        record_income(&mut w, b, &Harvest::of(&[6.0]));
        assert!(
            (w.agent(b).unwrap().income[0] - 3.0).abs() < 1e-12,
            "6 − (1 + 2)"
        );
    }

    #[test]
    fn with_two_goods_borrowers_borrow_each_good_they_lack() {
        let mut w = credit_world();
        add_goods(&mut w.config, 2);
        let b = spawn(&mut w, 2, 2);
        {
            let a = w.agent_mut(b).unwrap();
            a.holdings[1] = 4.0;
            a.income[1] = 5.0; // short of good 1 only; good 0 is at its endowment
        }
        let l = lender(&mut w);
        w.agent_mut(l).unwrap().holdings[1] = 40.0;
        borrow(&mut w, b);
        let loan = *w.loans().next().unwrap();
        assert_eq!((loan.good, loan.lender, loan.principal), (1, l, 6.0));
        assert_eq!(w.agent(b).unwrap().holdings[..2], [10.0, 10.0]);
        assert_eq!(w.agent(l).unwrap().holdings[..2], [40.0, 34.0]);
        w.tick = loan.due_tick;
        w.agent_mut(b).unwrap().holdings[1] = 20.0;
        settle(&mut w);
        assert_eq!(
            w.agent(b).unwrap().holdings[..2],
            [10.0, 8.0],
            "repaid in good 1"
        );
        assert_eq!(w.agent(l).unwrap().holdings[1], 46.0);
    }

    #[test]
    fn a_default_in_one_good_rolls_over_in_that_good() {
        let mut w = credit_world();
        add_goods(&mut w.config, 2);
        let b = spawn(&mut w, 2, 2);
        let l = lender(&mut w);
        w.agent_mut(b).unwrap().holdings[1] = 4.0;
        let id = w.originate_loan(l, b, 1, 6.0);
        let due_tick = w.loans().find(|x| x.id == id).unwrap().due_tick;
        w.tick = due_tick;
        settle(&mut w);
        assert_eq!(
            w.agent(b).unwrap().holdings[..2],
            [10.0, 2.0],
            "paid half its good 1"
        );
        let rolled = *w.loans().next().unwrap();
        assert_eq!(rolled.good, 1);
        assert!((rolled.principal - 10.0).abs() < 1e-12, "12 due − 2 paid");
        assert_eq!(w.events().defaults, 1);
    }

    #[test]
    fn income_is_per_good() {
        let mut w = credit_world();
        add_goods(&mut w.config, 2);
        let b = borrower(&mut w);
        let l = lender(&mut w);
        w.agent_mut(b).unwrap().metabolism[..2].copy_from_slice(&[2, 1]);
        w.originate_loan(l, b, 1, 5.0); // 10 due over 10 ticks in good 1
        record_income(&mut w, b, &Harvest::of(&[6.0, 3.0]));
        let income = w.agent(b).unwrap().income;
        assert!((income[0] - 4.0).abs() < 1e-12, "6 − 2, no good-0 debt");
        assert!((income[1] - 1.0).abs() < 1e-12, "3 − 1 − 1");
    }

    #[test]
    fn a_borrower_short_of_both_goods_borrows_both_from_the_same_neighbors() {
        let mut w = credit_world();
        add_goods(&mut w.config, 2);
        let b = borrower(&mut w); // good 0: 4 of 10, income 5
        {
            let a = w.agent_mut(b).unwrap();
            a.holdings[1] = 7.0;
            a.income[1] = 5.0;
        }
        let l = lender(&mut w);
        w.agent_mut(l).unwrap().holdings[1] = 40.0;
        borrow(&mut w, b);
        let goods: Vec<usize> = w.loans().map(|x| x.good).collect();
        assert_eq!(goods, vec![0, 1]);
        assert_eq!(w.agent(b).unwrap().holdings[..2], [10.0, 10.0]);
        assert_eq!(w.events().loans_made, 2);
    }
}
