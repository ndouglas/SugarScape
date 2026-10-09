//! Pure saved-record verification seams. These never construct or step a World.
use super::state::{Observation, TaskState};
use crate::{
    geometry::{Pos, Torus},
    rng::SimRng,
};
/// The production eligibility predicate, without reading physical stocks.
pub fn allowed(o: &Observation, s: &TaskState, site: u32, guarded: bool) -> bool {
    super::policy::allowed(o, s, site, guarded)
}
/// The production task selector, including its exact RNG tie draw.
pub fn select_target(o: &Observation, s: &TaskState, rng: &mut SimRng) -> Option<u32> {
    super::policy::select_target(o, s, rng)
}
/// The production chooser with a checked nonempty finite input contract.
pub fn choose(candidates: &[(Pos, u32, f64)], rng: &mut SimRng) -> Result<Pos, String> {
    if candidates.is_empty()
        || candidates
            .iter()
            .any(|c| c.0.x >= 11 || c.0.y >= 11 || !c.2.is_finite())
    {
        return Err("invalid verification candidates".into());
    }
    Ok(crate::rules::movement::choose(candidates, rng))
}
#[derive(Clone, Debug, PartialEq)]
pub struct VerifiedPlan {
    pub steps: Vec<(Pos, f64)>,
    pub cost: f64,
    pub expanded: usize,
}
/// Plan using the original Forage domain and original search. Input slots are
/// already ordered by the caller's recorded decision shortlist.
pub fn plan(sites: &[(Pos, f64)], goal: f64) -> Result<Option<VerifiedPlan>, String> {
    use crate::minds::goap::{
        self,
        forage::{Forage, PLAN_LIMIT},
        Domain,
    };
    if sites.is_empty()
        || sites.len() > 9
        || !goal.is_finite()
        || goal < 0.0
        || sites
            .iter()
            .any(|(p, v)| p.x >= 11 || p.y >= 11 || !v.is_finite() || *v < 0.0)
        || sites
            .iter()
            .enumerate()
            .any(|(i, s)| sites[..i].iter().any(|p| p.0 == s.0))
    {
        return Err("invalid verification domain".into());
    }
    let domain = Forage::new(Torus::new(11, 11), sites, goal);
    if !domain.is_goal(&(0, (1u16 << sites.len()) - 1)) {
        return Ok(None);
    }
    Ok(
        goap::plan(&domain, (0, 0), PLAN_LIMIT).map(|p| VerifiedPlan {
            steps: p.actions.iter().map(|i| sites[usize::from(*i)]).collect(),
            cost: p.cost,
            expanded: p.expanded,
        }),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn behavior_tree_verification_reuses_exact_domain_and_search() {
        use crate::minds::goap::{self, forage::Forage};
        let sites = [
            (Pos::new(2, 5), 0.0),
            (Pos::new(7, 5), 24.0),
            (Pos::new(4, 9), 24.0),
        ];
        let expected =
            goap::plan(&Forage::new(Torus::new(11, 11), &sites, 40.0), (0, 0), 4096).unwrap();
        let got = plan(&sites, 40.0).unwrap().unwrap();
        assert_eq!(
            (got.steps, got.cost, got.expanded),
            (
                expected
                    .actions
                    .iter()
                    .map(|i| sites[usize::from(*i)])
                    .collect(),
                expected.cost,
                expected.expanded
            )
        );
        assert!(plan(&sites, 60.0).unwrap().is_none());
        assert!(plan(&[], 20.0).is_err());
    }
    #[test]
    fn behavior_tree_verification_choose_preserves_exact_rng() {
        let c = [(Pos::new(3, 5), 1, 4.0), (Pos::new(2, 4), 1, 4.0)];
        let mut a = crate::rng::seeded(7);
        let mut b = a.clone();
        assert_eq!(
            choose(&c, &mut a).unwrap(),
            crate::rules::movement::choose(&c, &mut b)
        );
        assert_eq!(a, b);
        assert!(choose(&[], &mut a).is_err());
    }
}
