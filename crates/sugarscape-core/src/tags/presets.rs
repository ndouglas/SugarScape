//! The paper's runs and the replications' variants. Descriptions quote the
//! survey's measurements (20 seeds × 30 000 generations, recorded 2026-09-25).

use super::config::{DonationTest, InitialTolerance, Selection, TagsConfig, TieRule};
use crate::model::ModelConfig;
use crate::presets::ModelPreset;

fn preset(
    id: &'static str,
    name: &'static str,
    source: &'static str,
    description: &'static str,
    edit: impl FnOnce(&mut TagsConfig),
) -> ModelPreset {
    let mut c = TagsConfig::default();
    edit(&mut c);
    ModelPreset {
        id,
        name,
        source,
        description,
        config: ModelConfig::Tags(c),
    }
}

/// The paper's setup with ties going to the current agent, the only rule
/// that reproduces its tables (E&H).
fn published(c: &mut TagsConfig) {
    c.tie_rule = TieRule::Current;
}

const RCA: &str = "Riolo, Cohen & Axelrod 2001, Nature 414";
const EH: &str = "Edmonds & Hales 2003, JASSS 6(4)";

pub fn presets() -> Vec<ModelPreset> {
    vec![
        preset(
            "rca-published",
            "Published: ties to the current agent",
            RCA,
            "100 agents, each with a tag and a tolerance drawn from [0, 1], meet 3 others at random each generation and donate (cost 0.1, benefit 1) when the other's tag is within their tolerance of their own; each agent then faces a random other and the higher score has the offspring, whose tag is redrawn with probability 0.1 and whose tolerance gets Gaussian noise (s.d. 0.01) with probability 0.1. The paper does not say who wins a tie; Edmonds & Hales 2003 found only 'the current agent wins' matches its tables, so this preset uses it. Measured (20 seeds × 30 000 generations): donation 73.7% (the paper: 73.6%). The paper's cycle of clusters rising and being invaded is much slower than it describes: a median of 29 takeovers per run (Fig. 1 shows two in 500 generations), dominant clusters hold 86% of the agents (the paper: 75–80%) and are 91% one exact tag when they take over (the paper: 79%).",
            published,
        ),
        preset(
            "rca-literal",
            "Literal: ties by coin flip",
            EH,
            "The paper's rules as written: when two scores are equal, a coin flip decides who has the offspring. At 3 pairings this changes nothing (73.7%), but at 2 it gives 42% donation where the paper reports 4.3%, and at cost 0.5 it gives 45% where the paper reports 24.7% (20 seeds; Edmonds & Hales 2003 measured 42.6% and 45.9%). Open 'Published vs literal ties at P = 2' in the presets menu to run both.",
            |_| {},
        ),
        preset(
            "rca-published-p2",
            "Published, two pairings",
            RCA,
            "The published setup (ties to the current agent) with 2 pairings per generation: cooperation never takes hold. Measured: 2.0% donation (median of 20 seeds; the paper's Table 1: 4.3%).",
            |c| {
                published(c);
                c.pairings = 2;
            },
        ),
        preset(
            "rca-literal-p2",
            "Literal, two pairings",
            EH,
            "The literal rules (coin-flip ties) with 2 pairings per generation. Measured: 42% donation (median of 20 seeds; Edmonds & Hales 2003: 42.6%; the paper, with its unstated tie rule: 4.3%).",
            |c| c.pairings = 2,
        ),
        preset(
            "rca-strict",
            "Strict: donate only when |Δtag| < tolerance",
            EH,
            "The published setup, but a donor gives only when the tags differ by strictly less than its tolerance, so agents with identical tags and zero tolerance no longer have to help each other. Edmonds & Hales 2003 report no donation at all; measured, 1.4% (20 seeds) — mutation keeps a few tolerances above zero — against 73.7% with ≤.",
            |c| {
                published(c);
                c.donation_test = DonationTest::Below;
            },
        ),
        preset(
            "rs-no-forced-clones",
            "Tolerance may fall below zero",
            "Roberts & Sherratt 2002, Nature 418",
            "The published setup with the tolerance floor at −10⁻⁶ instead of 0, so an agent can refuse even an identical tag. Roberts & Sherratt 2002 reported 1.48% donation; measured, 1.4% (20 seeds). Cooperation in this model rests on forced donation between identical tags.",
            |c| {
                published(c);
                c.tolerance_floor = -1e-6;
            },
        ),
        preset(
            "eh-clones-only",
            "Tolerance fixed at zero",
            EH,
            "Every tolerance fixed at 0 and never mutated, with coin-flip ties: agents help only exact copies of their own tag. Measured: 75.3% donation (20 seeds; Edmonds & Hales 2003: 75.3%), more than with tolerance at work — the tolerance mechanism the paper credits does none of the work. With ties to the current agent the same setup gives 0%: from 100 distinct tags and equal scores, nobody ever has two offspring, so no two tags are ever equal.",
            |c| {
                c.initial_tolerance = InitialTolerance::Fixed(0.0);
                c.tolerance_mutation = 0.0;
            },
        ),
        preset(
            "eh-no-exact-clones",
            "No exact clones: tag noise",
            EH,
            "The published setup with Gaussian noise (s.d. 10⁻⁶) on every offspring's tag, so no two agents ever share a tag exactly and tolerance alone would have to sustain cooperation. It does not: 1.4% donation (20 seeds; Edmonds & Hales 2003: 1.5–1.9%).",
            |c| {
                published(c);
                c.tag_noise = 1e-6;
            },
        ),
        preset(
            "rca-adopt-p1",
            "Adopting better traits, one pairing",
            RCA,
            "The paper's learning variant: each agent compares itself with a random other and adopts its tag and tolerance 'with probability proportional to how much better the other agent is', here (s_other − s_self) / (b + c), the most one donation can move two scores apart — the paper gives no scale. With one pairing, measured: 48.8% donation (20 seeds; the paper: 49%). Scaled by the generation's range of scores instead it gives 7%.",
            |c| {
                c.selection = Selection::Adopt;
                c.pairings = 1;
            },
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn configs() -> Vec<(&'static str, TagsConfig)> {
        presets()
            .into_iter()
            .map(|p| match p.config {
                ModelConfig::Tags(c) => (p.id, c),
                _ => unreachable!(),
            })
            .collect()
    }

    #[test]
    fn presets_are_the_paper_plus_one_named_change() {
        let d = TagsConfig::default();
        let published = TagsConfig {
            tie_rule: TieRule::Current,
            ..d.clone()
        };
        for (id, c) in configs() {
            let expected = match id {
                "rca-published" => published.clone(),
                "rca-literal" => d.clone(),
                "rca-published-p2" => TagsConfig {
                    pairings: 2,
                    ..published.clone()
                },
                "rca-literal-p2" => TagsConfig {
                    pairings: 2,
                    ..d.clone()
                },
                "rca-strict" => TagsConfig {
                    donation_test: DonationTest::Below,
                    ..published.clone()
                },
                "rs-no-forced-clones" => TagsConfig {
                    tolerance_floor: -1e-6,
                    ..published.clone()
                },
                "eh-clones-only" => TagsConfig {
                    initial_tolerance: InitialTolerance::Fixed(0.0),
                    tolerance_mutation: 0.0,
                    ..d.clone()
                },
                "eh-no-exact-clones" => TagsConfig {
                    tag_noise: 1e-6,
                    ..published.clone()
                },
                "rca-adopt-p1" => TagsConfig {
                    selection: Selection::Adopt,
                    pairings: 1,
                    ..d.clone()
                },
                other => panic!("unexpected preset {other}"),
            };
            assert_eq!(c, expected, "{id}");
            assert!(c.validate().is_ok(), "{id}");
        }
        assert_eq!(configs().len(), 9);
    }
}
