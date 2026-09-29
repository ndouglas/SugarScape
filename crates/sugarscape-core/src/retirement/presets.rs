//! Axtell and Epstein's realizations, policy switch and sub-populations, and
//! the two readings that decide their results.

use super::config::{Counts, Groups, Policy, Renewal, RetirementConfig};
use crate::model::ModelConfig;
use crate::presets::ModelPreset;

const AE: &str = "Axtell & Epstein 1999, Brookings CSED WP 1";
const GSS: &str = "Epstein 2006, Generative Social Science, ch. 7";

fn preset(
    id: &'static str,
    name: &'static str,
    source: &'static str,
    description: &'static str,
    edit: impl FnOnce(&mut RetirementConfig),
) -> ModelPreset {
    let mut c = RetirementConfig::default();
    edit(&mut c);
    ModelPreset {
        id,
        name,
        source,
        description,
        config: ModelConfig::Retirement(c),
    }
}

pub fn presets() -> Vec<ModelPreset> {
    vec![
        preset(
            "ae-rapid",
            "Fig. 6-4: 15 % rational",
            GSS,
            "Axtell and Epstein's retirement model: 81 cohorts of 100 people aged 20 to 100, each with a random death age (U[60, 100]) and replaced at death by a 20-year-old. From 65 people may retire. 15 % are rational and retire at 65; 5 % retire at random (half each year); the other 80 % imitate: each has a network of 10 to 25 people within five years of its own age and retires once half its eligible friends have. The picture is theirs: one row per age, 20 at the top, rationals pink, imitators blue, randoms yellow, the retired red. 'Within the first 6 periods essentially all of the eligible population has retired.' Measured (20 seeds): 95 % of those eligible retired by period 7.7 on average — by period 6 in 4 runs of 20 (this one), by period 10 in all — rising steadily every time. Close: a period or two later than 'the first 6'.",
            |c| {
                c.rational = 0.15;
            },
        ),
        preset(
            "ae-base",
            "Table 6-1: the base case",
            AE,
            "Table 6-1's base case: 10 % rational, 85 % imitators, 5 % random, thresholds 0.5, networks of 10–25 people within up to five cohorts. Axtell and Epstein never define their 'transition time'; here it is the first period in which 95 % of the eligible have retired. Measured (20 seeds): 16 periods (± 3). Their other choices matter too: activating everyone in one random order instead of cohort by cohort, oldest first, it takes 26; counting every friend instead of the eligible ones, never.",
            |_| {},
        ),
        preset("ae-slow", "Fig. 6-5: 5 % rational", AE, "Figure 6-5: only 5 % rational, 90 % imitators. Retirement stalls at a fifth or so of those eligible, wavers — 'the trajectory is not monotone' — and then, once the retired old have spread their example, sweeps to 100 %: 'It is as if retirement percolates up from older to younger agents.' Measured (20 seeds): 61 periods (± 3); the first run climbs to 33 % by period 40, falls back to 20 %, and sweeps to 99 % at period 63. Reproduced.", |c| {
            c.rational = 0.05;
        }),
        preset(
            "ae-policy",
            "Animation 6-3: 65 to 62",
            AE,
            "Axtell and Epstein's policy experiment: retirement mandatory at 70, and once the age 65 norm is established, the eligibility age drops to 62, as Congress's did in 1961. The paper: the new norm 'emerges after twenty to thirty periods', and 'in about 35 periods if between 1 and 4 percent of the population responds rationally' — the sluggish response the model was built to explain. Measured (20 seeds): the new norm is reached 2 periods after the switch in every run, and in 2 or fewer at every rational share from 0 to 14 % (the ae-policy sweep, 10 seeds). Imitators just turned 62 count their eligible friends, who now include the retired 65-to-67-year-olds; half have retired, so they retire at once. The stated rules do not produce the decades.",
            |c| {
                c.rational = 0.05;
                c.mandatory = 70;
                c.policy = Policy {
                    enabled: true,
                    ..Policy::default()
                };
            },
        ),
        preset(
            "ae-groups",
            "Animation 6-4: two sub-populations",
            AE,
            "Figure 6-11's two sub-populations: every cohort split in half, rationals (10 %) only in the second half, and each network drawing 10 % of its members from the other half. Axtell and Epstein: this 'loose coupling is sufficient for the group containing some rationals to pull the other into conformity'. Measured (the ae-coupling sweeps, 10 seeds): the group without rationals reaches the norm at period 46 instead of 75 — pulled in, as they say — but the group with rationals slows from 19 to 34, and at couplings of 0.2 and more both take about 58. The coupling pulls both ways; in their figure the rational group stays fast.",
            |c| {
                c.groups = Groups {
                    enabled: true,
                    ..Groups::default()
                };
            },
        ),
        preset(
            "ae-all-members",
            "Footnote 5: counting every member",
            AE,
            "Footnote 5: 'It makes a difference to the numerical results whether an agent considers all agents in its social network, or only those who are eligible to retire. However, the qualitative character of the results … do not depend on this distinction.' Here imitators count every friend. A network spans up to five years either side, so the young friends who cannot retire hold the share below one half, and imitators never retire. Measured (20 seeds, 600 periods): no norm, ever; only the rationals and randoms retire.",
            |c| {
                c.counts = Counts::All;
            },
        ),
        preset(
            "ae-replace",
            "Friends replaced, 5 % rational",
            AE,
            "A choice the paper never states: when a friend dies, is its place in the network taken by the 20-year-old reborn in its slot (the default; their pseudo-code reuses agent objects) or does the agent find a replacement of about its own age (here)? With friends replaced, networks keep their eligible members and imitation is harder: 5 % rationality never establishes the norm (10 seeds, 600 periods), and 10 % reaches it in 8 runs of 10, after 22 to 279 periods, while the other 2 never do within 600 — the 'minimum proportions … rational' and the rapidly growing variance of Axtell and Epstein's Figure 6-6, which the default reading does not produce.",
            |c| {
                c.rational = 0.05;
                c.renewal = Renewal::Replace;
            },
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_set_what_they_say() {
        let got: Vec<(&str, RetirementConfig)> = presets()
            .into_iter()
            .map(|p| match p.config {
                ModelConfig::Retirement(c) => (p.id, c),
                _ => panic!("{} is not a retirement preset", p.id),
            })
            .collect();
        let find = |id| got.iter().find(|(i, _)| *i == id).unwrap().1.clone();
        assert_eq!(find("ae-base"), RetirementConfig::default());
        assert_eq!(find("ae-rapid").rational, 0.15);
        let p = find("ae-policy");
        assert_eq!((p.mandatory, p.policy.enabled, p.policy.to), (70, true, 62));
        assert_eq!(find("ae-all-members").counts, Counts::All);
        assert_eq!(find("ae-replace").renewal, Renewal::Replace);
        assert!(got.iter().all(|(_, c)| c.validate().is_ok()));
    }
}
