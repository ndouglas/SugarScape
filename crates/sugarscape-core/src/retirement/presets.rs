//! Axtell and Epstein's realizations, policy switch and sub-populations, and
//! explicit reconstruction choices and operational first-crossing diagnostics.

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
            "15 % rational,80 % imitators,5 % random, networks initially drawn from nearby birth cohorts. Eligible imitators retire when at least half their eligible friends have. Native 50-run audit: mean first 95 % crossing 7.78 periods; source prose 15 % and caption 20 % differ and describe one six-period realization. Slot newborn-pointer renewal and oldest-cohort-first traversal are reconstruction choices; first 95 % crossing is not a persistent age norm.",
            |c| {
                c.rational = 0.15;
            },
        ),
        preset(
            "ae-base",
            "Table 6-1: the base case",
            AE,
            "Table 6-1 base:10 % rational,85 % imitators,5 % random, threshold .5, network size 10–25, extent up to five birth cohorts. Native 50-run first 95 % crossing mean 16.60±3.76. First95 is an operational eligible-retired proxy; the papers do not define their age-norm stopping rule. Slot renewal and oldest-first traversal are explicit reconstruction choices.",
            |_| {},
        ),
        preset("ae-slow", "Fig. 6-5: 5 % rational", AE, "Figure 6-5:5 % rational and 90 % imitators. Retirement wavers before cascading. Native 50-run first 95 % crossing mean 61.54±3.92; this supports qualitative shape, not the displayed source plateau near 375 or perfect absorption. Initial age proximity can change under Slot newborn-pointer renewal.", |c| {
            c.rational = 0.05;
        }),
        preset(
            "ae-policy",
            "Animation 6-3: 65 to 62",
            AE,
            "Original AE policy setup:5 % rational,5 % random, homogeneous threshold .5, mandatory 70, eligibility 65→62 at the first aggregate proxy crossing. Native 50-run post-switch first 95 % crossing is about 2 periods. This does not demonstrate a pre-switch age 65 norm. Revised GSS uses thresholds U[.5,1] (mean .75, SD .14433756729740646): a separate 50-run diagnostic reaches 22/50 within 100, conditional 43.73±32.90. Source norm termination is undefined; neither first 95 % crossing nor a short-lived mode proves persistent norm establishment.",
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
            "Each cohort split in half: rational share 10 % within B,0 % in A, expected 5 % globally. Each edge independently draws the other group with probability.1. On source coupling .05→.20, native 50-run B first 95 % crossing slows 24.72→57.90 while A converges 64.06→58.24. The source figure also slows the rational group. First 95 tests trend shape, not exact age-norm reproduction.",
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
            "Footnote 5 denominator alternative: imitate using every network member. Native 50 Slot runs over 600 attain no first 95 % crossing, but imitators do retire (mean 91.48 events), and rolling event mode 65 can arise from a small minority. The source leaves its qualitative norm undefined, so this proxy difference is a reconstruction-dependent diagnostic, not a categorical refutation.",
            |c| {
                c.counts = Counts::All;
            },
        ),
        preset(
            "ae-replace",
            "Friends replaced, 5 % rational",
            AE,
            "Renewal sensitivity: replace dead friends with peers drawn within the holder’s extent, preserving the deceased link’s group category. Slot instead passes the place to a newborn. Neither is uniquely specified by the source pseudocode; fixed-identity alternatives are also possible. Native 10 %-rational comparison over 600: eligible counting reaches 37/50, all-member counting 3/50. These observations do not identify an infinite-time critical rational share; this preset uses 5 % rational.",
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
