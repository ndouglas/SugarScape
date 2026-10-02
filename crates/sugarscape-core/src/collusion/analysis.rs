//! What a session learned: the limit cycle its strategies settle into
//! (the code's `ConvergenceResults`), the profit gain Δ, whether the
//! strategies are an equilibrium (by either reading), the responses to
//! deviations (the code's `computeIndividualIR`, and the critics'), den Boer,
//! Meylahn & Schinkel's RP-completeness, and re-pairing firms from different
//! sessions.

use serde::Serialize;

use super::config::{BestResponseTo, EquilibriumCheck};
use super::demand::Game;
use super::learner::Space;

/// Periods an impulse response follows.
pub const HORIZON: usize = 25;

/// Each firm's greedy price in each state: `strategies[i][s]`.
pub type Strategies = Vec<Vec<u8>>;

fn play(strategies: &Strategies, s: usize) -> Vec<u8> {
    strategies.iter().map(|f| f[s]).collect()
}

/// The deterministic path from a state and the cycle it ends in.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Cycle {
    /// The cycle's states, in order: the state after each period's prices.
    pub states: Vec<usize>,
    /// The prices played into each cycle state.
    pub actions: Vec<Vec<u8>>,
    /// Each firm's mean profit over the cycle.
    pub profits: Vec<f64>,
    /// Each firm's mean price over the cycle.
    pub prices: Vec<f64>,
}

impl Cycle {
    pub fn len(&self) -> usize {
        self.states.len()
    }

    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }

    /// Δ per firm.
    pub fn gains(&self, game: &Game) -> Vec<f64> {
        (0..game.firms)
            .map(|i| game.gain(i, self.profits[i]))
            .collect()
    }

    /// The session's Δ: the firms' mean.
    pub fn gain(&self, game: &Game) -> f64 {
        let g = self.gains(game);
        g.iter().sum::<f64>() / g.len() as f64
    }

    /// A single price profile repeated: every firm at one price.
    pub fn is_point(&self) -> bool {
        self.len() == 1
    }
}

/// Replays `strategies` from `start` without exploration until a state
/// repeats; the repeated stretch is the limit cycle.
pub fn limit_cycle(game: &Game, space: &Space, strategies: &Strategies, start: usize) -> Cycle {
    let mut seen: Vec<usize> = Vec::new();
    let mut acts: Vec<Vec<u8>> = Vec::new();
    let mut s = start;
    loop {
        let a = play(strategies, s);
        s = space.next(s, &a);
        if let Some(k) = seen.iter().position(|&x| x == s) {
            // The state was first reached at k: the cycle is k.. plus this
            // period, which re-enters it.
            let mut states = seen[k + 1..].to_vec();
            states.push(s);
            let mut actions = acts[k + 1..].to_vec();
            actions.push(a);
            let len = actions.len() as f64;
            let profits = (0..game.firms)
                .map(|i| actions.iter().map(|x| game.profit(x, i)).sum::<f64>() / len)
                .collect();
            let prices = (0..game.firms)
                .map(|i| {
                    actions
                        .iter()
                        .map(|x| game.grid[i][usize::from(x[i])])
                        .sum::<f64>()
                        / len
                })
                .collect();
            return Cycle {
                states,
                actions,
                profits,
                prices,
            };
        }
        seen.push(s);
        acts.push(a);
    }
}

/// Firm `i`'s values when everyone follows `strategies`: V(s) = π(σ(s)) +
/// δ V(next), exact on each path's cycle (the code's `computeQCell`).
pub fn policy_values(
    game: &Game,
    space: &Space,
    strategies: &Strategies,
    i: usize,
    delta: f64,
) -> Vec<f64> {
    let mut v = vec![f64::NAN; space.states];
    for start in 0..space.states {
        if !v[start].is_nan() {
            continue;
        }
        let mut path = Vec::new();
        let mut s = start;
        while v[s].is_nan() && !path.contains(&s) {
            path.push(s);
            s = space.next(s, &play(strategies, s));
        }
        let mut tail = if v[s].is_nan() {
            // s is on the path: a new cycle from its position.
            let k = path.iter().position(|&x| x == s).unwrap();
            let cycle = &path[k..];
            let rewards: Vec<f64> = cycle
                .iter()
                .map(|&x| game.profit(&play(strategies, x), i))
                .collect();
            let l = cycle.len();
            let dl = delta.powi(l as i32);
            for (j, &x) in cycle.iter().enumerate() {
                let mut sum = 0.0;
                for t in 0..l {
                    sum += delta.powi(t as i32) * rewards[(j + t) % l];
                }
                v[x] = sum / (1.0 - dl);
            }
            path.truncate(k);
            v[s]
        } else {
            v[s]
        };
        for &x in path.iter().rev() {
            tail = game.profit(&play(strategies, x), i) + delta * tail;
            v[x] = tail;
        }
    }
    v
}

/// Firm `i`'s Q(s, a) for each state and price when the rivals follow
/// `strategies` and firm `i` then plays as `values` says.
fn q_against(
    game: &Game,
    space: &Space,
    strategies: &Strategies,
    i: usize,
    delta: f64,
    values: &[f64],
) -> Vec<f64> {
    let m = game.prices;
    let mut q = vec![0.0; space.states * m];
    for s in 0..space.states {
        let mut a = play(strategies, s);
        for p in 0..m as u8 {
            a[i] = p;
            q[s * m + usize::from(p)] = game.profit(&a, i) + delta * values[space.next(s, &a)];
        }
    }
    q
}

/// Firm `i`'s optimal values against the rivals' `strategies`, by value
/// iteration to 1e-13.
pub fn optimal_values(
    game: &Game,
    space: &Space,
    strategies: &Strategies,
    i: usize,
    delta: f64,
) -> Vec<f64> {
    let m = game.prices;
    let mut v = vec![0.0; space.states];
    loop {
        let q = q_against(game, space, strategies, i, delta, &v);
        let mut moved: f64 = 0.0;
        for s in 0..space.states {
            let best = q[s * m..(s + 1) * m]
                .iter()
                .copied()
                .fold(f64::NEG_INFINITY, f64::max);
            moved = moved.max((best - v[s]).abs());
            v[s] = best;
        }
        if moved < 1e-13 || delta == 0.0 {
            return v;
        }
    }
}

/// Whether each firm's strategy is a best response in each state, by
/// `check`: `best[i][s]`.
pub fn best_responses(
    game: &Game,
    space: &Space,
    strategies: &Strategies,
    delta: f64,
    check: EquilibriumCheck,
) -> Vec<Vec<bool>> {
    let m = game.prices;
    (0..game.firms)
        .map(|i| {
            let v = match check {
                EquilibriumCheck::OneShot => policy_values(game, space, strategies, i, delta),
                EquilibriumCheck::BestResponse => optimal_values(game, space, strategies, i, delta),
            };
            let q = q_against(game, space, strategies, i, delta, &v);
            (0..space.states)
                .map(|s| {
                    let row = &q[s * m..(s + 1) * m];
                    let top = row.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                    top - row[usize::from(strategies[i][s])] <= 1e-10 * top.abs().max(1.0)
                })
                .collect()
        })
        .collect()
}

/// Equilibrium flags: on the limit path, off it, and everywhere — each the
/// share of states where every firm best responds, and whether that share is 1.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Equilibrium {
    pub on_path: bool,
    pub off_path_share: f64,
    pub all_share: f64,
}

pub fn equilibrium(
    game: &Game,
    space: &Space,
    strategies: &Strategies,
    cycle: &Cycle,
    delta: f64,
    check: EquilibriumCheck,
) -> Equilibrium {
    let best = best_responses(game, space, strategies, delta, check);
    let all = |s: usize| best.iter().all(|f| f[s]);
    let on_path = cycle.states.iter().all(|&s| all(s));
    let off: Vec<usize> = (0..space.states)
        .filter(|s| !cycle.states.contains(s))
        .collect();
    let share = |set: &[usize]| {
        if set.is_empty() {
            1.0
        } else {
            set.iter().filter(|&&s| all(s)).count() as f64 / set.len() as f64
        }
    };
    let every: Vec<usize> = (0..space.states).collect();
    Equilibrium {
        on_path,
        off_path_share: share(&off),
        all_share: share(&every),
    }
}

/// One deviation and what followed.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Response {
    /// The deviating firm and the cycle position it deviated from.
    pub firm: usize,
    pub position: usize,
    /// Each firm's price index before the deviation (the cycle's prices into
    /// the deviation state) and at the deviation.
    pub before: Vec<u8>,
    pub deviation: u8,
    /// Prices (grid indices) in the deviation period and the `HORIZON` − 1
    /// after: `path[t][j]`.
    pub path: Vec<Vec<u8>>,
    /// Periods until the state is back on the pre-deviation cycle (the
    /// code's `PunishmentStrategy`), if it returns within the horizon.
    pub returned: Option<usize>,
    /// Periods until play enters a cycle — the pre-deviation one or a new
    /// one (the code's `ShockLength`, Table A5's punishment length).
    pub settled: usize,
}

impl Response {
    /// The punishment-like response the survey fixed: in the period after
    /// the deviation every other firm's price is at least one grid step
    /// below what it charged in the deviation period, and the state is back
    /// on the cycle within `HORIZON` periods.
    pub fn punishment_like(&self) -> bool {
        let (dev, next) = (&self.path[0], &self.path[1]);
        (0..dev.len())
            .filter(|&j| j != self.firm)
            .all(|j| next[j] < dev[j])
            && self.returned.is_some()
    }

    /// Firm j's relative price change from the deviation period to the next
    /// (Epivent & Lambin's measure).
    pub fn change(&self, game: &Game, j: usize) -> f64 {
        let (a, b) = (
            game.grid[j][usize::from(self.path[0][j])],
            game.grid[j][usize::from(self.path[1][j])],
        );
        (b - a) / a
    }
}

/// A one-period deviation by `firm` to `price` from cycle position `k`: the
/// deviation period's other prices are the strategies' at the cycle state,
/// then everyone follows the strategies (the code's `computeIndividualIR`).
pub fn deviate(
    space: &Space,
    strategies: &Strategies,
    cycle: &Cycle,
    k: usize,
    firm: usize,
    price: u8,
) -> Response {
    let start = cycle.states[k];
    let before = play(strategies, start);
    let mut a = before.clone();
    a[firm] = price;
    let mut path = Vec::with_capacity(HORIZON);
    let mut seen: Vec<usize> = Vec::new();
    let mut s = start;
    let mut returned = None;
    let mut settled = None;
    // As the code's computeIndividualIR: the first period the state is on
    // the pre-deviation cycle, or else where a new cycle begins.
    for t in 1..=HORIZON.max(space.states + 1) {
        s = space.next(s, &a);
        if t <= HORIZON {
            path.push(a.clone());
        }
        if settled.is_none() {
            if cycle.states.contains(&s) {
                settled = Some(t);
            } else if let Some(first) = seen.iter().position(|&x| x == s) {
                settled = Some(first + 1);
            }
        }
        if returned.is_none() && t <= HORIZON && cycle.states.contains(&s) {
            returned = Some(t);
        }
        seen.push(s);
        if settled.is_some() && t >= HORIZON {
            break;
        }
        a = play(strategies, s);
    }
    Response {
        firm,
        position: k,
        before,
        deviation: price,
        path,
        returned,
        settled: settled.expect("a deterministic path enters a cycle"),
    }
}

/// The paper's deviation: one period at the static best response — to the
/// rivals' prices at the cycle state (`path`), or at the state the code
/// passes, the one numbered by the cycle position (`code`).
pub fn best_response_deviation(
    game: &Game,
    space: &Space,
    strategies: &Strategies,
    cycle: &Cycle,
    k: usize,
    firm: usize,
    to: BestResponseTo,
) -> Response {
    let against = match to {
        BestResponseTo::Path => cycle.states[k],
        BestResponseTo::Code => k.min(space.states - 1),
    };
    let br = game.static_best_response(&play(strategies, against), firm);
    deviate(space, strategies, cycle, k, firm, br)
}

/// Every one-period deviation to every other price, by every firm, from
/// every cycle position (CCDP-A; Epivent & Lambin's Table 1).
pub fn every_deviation(space: &Space, strategies: &Strategies, cycle: &Cycle) -> Vec<Response> {
    let mut out = Vec::new();
    for k in 0..cycle.len() {
        let on = play(strategies, cycle.states[k]);
        for (firm, &own) in on.iter().enumerate() {
            for p in 0..space.prices as u8 {
                if p != own {
                    out.push(deviate(space, strategies, cycle, k, firm, p));
                }
            }
        }
    }
    out
}

/// den Boer, Meylahn & Schinkel's RP-completeness: every one-period
/// deviation, by either firm to any other price, from every cycle state,
/// draws a punishment-like response.
pub fn rp_complete(space: &Space, strategies: &Strategies, cycle: &Cycle) -> bool {
    every_deviation(space, strategies, cycle)
        .iter()
        .all(Response::punishment_like)
}

/// Epivent & Lambin's invitation from a point cycle: `firm` one step up for
/// `hold` + 1 periods, every other firm forced to match from the second
/// period on; returns the deviator's price index when it regains control and
/// its price before, or `None` if the cycle is not a point or it is at the
/// top of the grid.
pub fn invitation(
    game: &Game,
    space: &Space,
    strategies: &Strategies,
    cycle: &Cycle,
    firm: usize,
    hold: u32,
) -> Option<(u8, u8)> {
    if !cycle.is_point() {
        return None;
    }
    let mut s = cycle.states[0];
    let before = play(strategies, s)[firm];
    if usize::from(before) + 1 >= game.prices {
        return None;
    }
    let up = before + 1;
    let raised = game.grid[firm][usize::from(up)];
    for t in 0..=hold {
        let mut a = play(strategies, s);
        a[firm] = up;
        if t >= 1 {
            for (j, x) in a.iter_mut().enumerate() {
                if j != firm {
                    *x = game.nearest(j, raised);
                }
            }
        }
        s = space.next(s, &a);
    }
    Some((play(strategies, s)[firm], before))
}

/// Lambin's (2024) Theorem 1 for two symmetric firms: rank prices by π̃,
/// the profit against a uniformly random rival; after long exploration the
/// firms try them in that order, keeping the best symmetric profit found,
/// until it beats (1 − δ)π̃(next) + δπ̃(first). Returns the price index they
/// settle on (his eq. 6).
pub fn lambin_point(game: &Game, delta: f64) -> u8 {
    let m = game.prices;
    let tilde: Vec<f64> = (0..m as u8)
        .map(|a| (0..m as u8).map(|q| game.profit(&[a, q], 0)).sum::<f64>() / m as f64)
        .collect();
    let mut order: Vec<u8> = (0..m as u8).collect();
    order.sort_by(|&a, &b| tilde[usize::from(b)].total_cmp(&tilde[usize::from(a)]));
    let top = tilde[usize::from(order[0])];
    let mut best = (order[0], game.profit(&[order[0], order[0]], 0));
    for j in 0..m - 1 {
        let tried = order[j];
        let p = game.profit(&[tried, tried], 0);
        if p > best.1 {
            best = (tried, p);
        }
        let next = usize::from(order[j + 1]);
        if best.1 > (1.0 - delta) * tilde[next] + delta * top {
            return best.0;
        }
    }
    best.0
}

/// Pairs firm 0's strategy from one session with firm 1's from another
/// (Eschenbaum, Mellgren & Zahn; two firms only) and replays from `start`.
pub fn repair(
    game: &Game,
    space: &Space,
    first: &Strategies,
    second: &Strategies,
    start: usize,
) -> Cycle {
    let mixed: Strategies = vec![first[0].clone(), second[1].clone()];
    limit_cycle(game, space, &mixed, start)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collusion::config::CollusionConfig;

    fn setup() -> (Game, Space) {
        let c = CollusionConfig::default();
        (Game::new(&c), Space::of(&c))
    }

    /// Both firms at `high` while both were at `high` last period; otherwise
    /// at `low` (grim trigger on the grid).
    fn grim(space: &Space, high: u8, low: u8) -> Strategies {
        let both_high = space.encode(&[vec![high, high]]);
        let one = (0..space.states)
            .map(|s| if s == both_high { high } else { low })
            .collect::<Vec<u8>>();
        vec![one.clone(), one]
    }

    #[test]
    fn a_constant_strategy_cycles_at_one_point() {
        let (game, space) = setup();
        let strategies = vec![vec![8u8; 225], vec![8u8; 225]];
        let c = limit_cycle(&game, &space, &strategies, 0);
        assert!(c.is_point());
        assert_eq!(c.states, vec![space.encode(&[vec![8, 8]])]);
        assert!((c.gain(&game) - 0.794).abs() < 0.001);
    }

    #[test]
    fn an_alternating_pair_cycles_in_two() {
        let (game, space) = setup();
        // Each firm plays the other's last price, starting from (3, 9).
        let s: Vec<u8> = (0..225).map(|x| (x % 15) as u8).collect();
        let t: Vec<u8> = (0..225).map(|x| (x / 15) as u8).collect();
        let c = limit_cycle(&game, &space, &vec![s, t], space.encode(&[vec![3, 9]]));
        assert_eq!(c.len(), 2);
        assert_eq!(c.actions, vec![vec![3, 9], vec![9, 3]]);
        let mean = (game.grid[0][3] + game.grid[0][9]) / 2.0;
        assert!((c.prices[0] - mean).abs() < 1e-12);
    }

    #[test]
    fn grim_trigger_at_monopoly_is_an_equilibrium_and_rp_complete() {
        let (game, space) = setup();
        let g = grim(&space, 12, 1);
        let c = limit_cycle(&game, &space, &g, space.encode(&[vec![12, 12]]));
        assert!(c.is_point());
        for check in [EquilibriumCheck::BestResponse, EquilibriumCheck::OneShot] {
            let e = equilibrium(&game, &space, &g, &c, 0.95, check);
            assert!(e.on_path, "{check:?}");
        }
        // Too impatient to sustain it.
        let e = equilibrium(&game, &space, &g, &c, 0.1, EquilibriumCheck::BestResponse);
        assert!(!e.on_path);
        // A deviation is punished forever, so it never returns: not
        // punishment-like by the survey's rule (no return within 25 periods).
        let r = deviate(&space, &g, &c, 0, 0, 5);
        assert_eq!(r.path[1], vec![1, 1]);
        assert_eq!(r.returned, None);
        // A new cycle at (1, 1), entered in period 2.
        assert_eq!(r.settled, 2);
        assert!(!rp_complete(&space, &g, &c));
    }

    #[test]
    fn a_one_period_punishment_is_punishment_like_and_rp_complete() {
        let (game, space) = setup();
        // At (12, 12) both stay; after anything else both play 1 for a period
        // and then return to 12: from (1, 1) back to 12.
        let cartel = space.encode(&[vec![12, 12]]);
        let punish = space.encode(&[vec![1, 1]]);
        let one: Vec<u8> = (0..225)
            .map(|s| if s == cartel || s == punish { 12 } else { 1 })
            .collect();
        let st = vec![one.clone(), one];
        let c = limit_cycle(&game, &space, &st, cartel);
        assert!(c.is_point());
        let r = deviate(&space, &st, &c, 0, 0, 3);
        assert_eq!(r.path[0], vec![3, 12]);
        assert_eq!(r.path[1], vec![1, 1]);
        assert_eq!((r.returned, r.settled), (Some(3), 3));
        assert!(r.punishment_like());
        assert!(r.change(&game, 1) < 0.0);
        assert!(rp_complete(&space, &st, &c));
    }

    #[test]
    fn the_codes_best_response_answers_state_one() {
        let (game, space) = setup();
        // Rivals play 14 at state 0, 8 elsewhere.
        let mut one = vec![8u8; 225];
        one[0] = 14;
        let st = vec![one.clone(), one];
        let c = limit_cycle(&game, &space, &st, space.encode(&[vec![8, 8]]));
        let path = best_response_deviation(&game, &space, &st, &c, 0, 0, BestResponseTo::Path);
        let code = best_response_deviation(&game, &space, &st, &c, 0, 0, BestResponseTo::Code);
        assert_eq!(path.deviation, game.static_best_response(&[8, 8], 0));
        assert_eq!(code.deviation, game.static_best_response(&[14, 14], 0));
        assert_ne!(path.deviation, code.deviation);
    }

    #[test]
    fn an_invitation_ends_where_the_strategy_sends_the_deviator() {
        let (game, space) = setup();
        let st = vec![vec![8u8; 225], vec![8u8; 225]];
        let c = limit_cycle(&game, &space, &st, 0);
        assert_eq!(invitation(&game, &space, &st, &c, 0, 5), Some((8, 8)));
        let top = vec![vec![14u8; 225], vec![14u8; 225]];
        let ct = limit_cycle(&game, &space, &top, 0);
        assert_eq!(invitation(&game, &space, &top, &ct, 0, 5), None);
    }

    #[test]
    fn lambin_s_theorem_one_on_calvano_s_grid() {
        let (game, _) = setup();
        // The reader's computation (reading notes §5): I = 1.73770 at δ = 0.95,
        // 1.69895 at δ = 0; the first price tried is π̃'s best, 1.58271.
        assert_eq!(lambin_point(&game, 0.95), 8);
        assert_eq!(lambin_point(&game, 0.0), 7);
        assert!((game.grid[0][8] - 1.7377021).abs() < 1e-6);
    }

    #[test]
    fn policy_values_sum_the_discounted_cycle() {
        let (game, space) = setup();
        let st = vec![vec![8u8; 225], vec![8u8; 225]];
        let v = policy_values(&game, &space, &st, 0, 0.9);
        let pi = game.profit(&[8, 8], 0);
        assert!(v.iter().all(|x| (x - pi / 0.1).abs() < 1e-9));
        let opt = optimal_values(&game, &space, &st, 0, 0.9);
        let br = game.static_best_response(&[8, 8], 0);
        assert!(opt
            .iter()
            .all(|x| (x - game.profit(&[br, 8], 0) / 0.1).abs() < 1e-9));
        let repaired = repair(&game, &space, &st, &st, 0);
        assert!(repaired.is_point());
    }
}
