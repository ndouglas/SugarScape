//! Local mutual-defection battles with no resource damage.
use super::*;
use super::{
    types::{bump, Claim},
    world::Engine,
};
impl Engine {
    pub(crate) fn combat(&mut self, c: &DemocraticPeaceConfig) -> Result<Vec<Claim>, String> {
        let keys: Vec<_> = self.fronts.keys().copied().collect();
        let mut claims = Vec::new();
        for k in keys {
            let acts = self.fronts[&k].actions;
            if !acts.iter().any(|a| *a) {
                continue;
            }
            let valid = self.fronts[&k].path.is_some_and(|p| {
                self.cells[p[0]].owner == k[0]
                    && self.cells[p[1]].owner == k[1]
                    && self.adjacent(p[0]).contains(&p[1])
            });
            if !valid {
                let actor = k[usize::from(!acts[0])];
                let target = if actor == k[0] { k[1] } else { k[0] };
                let p = self.path(actor, target)?;
                let f = self.fronts.get_mut(&k).unwrap();
                f.path = Some(p);
                f.path_proposer = Some(actor);
            }
            if !(acts[0] && acts[1]) {
                continue;
            }
            bump(&mut self.counters.mutual_d_front_periods)?;
            let v = self.fronts[&k].commitments;
            let q = [
                super::resources::probability(
                    v[0],
                    v[1],
                    c.victory_threshold,
                    c.victory_exponent,
                    c.probability_direction,
                    c.zero_ratio,
                )?,
                super::resources::probability(
                    v[1],
                    v[0],
                    c.victory_threshold,
                    c.victory_exponent,
                    c.probability_direction,
                    c.zero_ratio,
                )?,
            ];
            let success = if c.opposing_victories == OpposingVictories::IndependentClaims {
                [
                    super::world::chance(q[0], &mut self.rng),
                    super::world::chance(q[1], &mut self.rng),
                ]
            } else {
                let scale = (q[0] + q[1]).max(1.);
                let r = super::world::draw(&mut self.rng);
                [
                    r < q[0] / scale,
                    r >= q[0] / scale && r < (q[0] + q[1]) / scale,
                ]
            };
            let victory = success.iter().any(|a| *a);
            let stalemate =
                !victory && super::world::chance(c.stalemate_probability, &mut self.rng);
            let f = self.fronts.get_mut(&k).unwrap();
            f.victory_probabilities = [Some(q[0]), Some(q[1])];
            f.victory_ratios = [Some(Ratio::new(v[0], v[1])), Some(Ratio::new(v[1], v[0]))];
            f.claims = success;
            if victory {
                bump(&mut self.counters.completed_victory_battles)?;
                if success[0] && success[1] {
                    bump(&mut self.counters.opposing_claims)?;
                }
                for (i, &won) in success.iter().enumerate() {
                    if won {
                        claims.push(Claim {
                            states: k,
                            side: i,
                            path: f.path.unwrap(),
                        });
                    }
                }
            }
            if stalemate {
                bump(&mut self.counters.completed_stalemate_battles)?;
            }
            if victory || stalemate {
                f.actions = [false; 2];
                f.path = None;
                f.path_proposer = None;
            }
        }
        Ok(claims)
    }
}
