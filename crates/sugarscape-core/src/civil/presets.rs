//! The paper's runs (Table 2) and scenarios, and NetLogo's defaults.

use super::config::{CivilConfig, Jail, Quirks, Ramp, Variant, Vision};
use crate::config::ScheduledChange;
use crate::model::ModelConfig;
use crate::presets::ModelPreset;

fn preset(
    id: &'static str,
    name: &'static str,
    source: &'static str,
    description: &'static str,
    edit: impl FnOnce(&mut CivilConfig),
) -> ModelPreset {
    let mut c = CivilConfig::default();
    edit(&mut c);
    ModelPreset {
        id,
        name,
        source,
        description,
        config: ModelConfig::Civil(c),
    }
}

/// A Model I column of Table 2 — cop and agent vision, legitimacy, J_max
/// (None: terms never end), movement, initial cop density — with the arrest
/// ratio rounded down, without which the paper's rule never rebels (the
/// spec's Finding).
fn model_one(
    c: &mut CivilConfig,
    vision: f64,
    legitimacy: f64,
    jail: Option<u32>,
    movement: bool,
    cops: f64,
) {
    c.variant = Variant::Rebellion;
    c.vision = Vision {
        agent: vision,
        cop: vision,
    };
    c.legitimacy = legitimacy;
    c.jail = match jail {
        Some(max) => Jail {
            max,
            infinite: false,
        },
        None => Jail {
            max: 30,
            infinite: true,
        },
    };
    c.movement = movement;
    c.cop_density = cops;
    c.quirks.floor_ratio = true;
}

/// A Model II column of Table 2: vision 1.7, J_max 15, random movement,
/// the given legitimacy and initial cop density, the paper's arrest rule.
fn model_two(c: &mut CivilConfig, legitimacy: f64, cops: f64) {
    c.variant = Variant::Ethnic;
    c.vision = Vision {
        agent: 1.7,
        cop: 1.7,
    };
    c.legitimacy = legitimacy;
    c.jail = Jail {
        max: 15,
        infinite: false,
    };
    c.cop_density = cops;
}

fn at(tick: u64, path: &str, value: serde_json::Value) -> ScheduledChange {
    ScheduledChange {
        tick,
        set: [(path.to_string(), value)].into_iter().collect(),
    }
}

const SOURCE: &str = "Epstein 2002, PNAS 99 suppl. 3";

pub fn presets() -> Vec<ModelPreset> {
    use serde_json::json;
    vec![
        preset(
            "cv-run-1-no-movement",
            "Run 1 — no movement",
            SOURCE,
            "Table 2's Run 1: vision 1.7 (the eight neighbors), L = 0.89, J_max = 15, cops 0.04, and agents that do not move (cops still do), with C/A rounded down — without it the paper's stated rule gives Model I no rebellion (the sweep cv-ratio-rules). The paper shows this society's deceptive agents and local outbursts (Figs. 1–2): agents turn red when the cops near them move on. Measured (seeds 1–20): a steady simmer of 33–62 actives (mean 48 over t = 100–700) with no calm stretches.",
            |c| model_one(c, 1.7, 0.89, Some(15), false, 0.04),
        ),
        preset(
            "cv-run-2-punctuated",
            "Run 2 — punctuated equilibrium",
            SOURCE,
            "Table 2's Run 2 (vision 7, L = 0.82, J_max = 30, cops 0.04) with C/A rounded down, as NetLogo's Rebellion does. With the paper's stated P = 1 − exp(−k·C/A) no seed of 20 has an outburst in 3000 ticks (at most 13–34 actives): a crowd that outnumbers the cops still faces P ≈ 0.5, which outweighs every grievance at L = 0.82. Rounded down, P is 0 once actives outnumber the cops in view, and the paper's punctuated equilibrium appears (Figs. 3–4). Measured (seeds 1–20, 3000 ticks): 101–122 outbursts above 50 actives, peaks of 290–368, fewer than 10 actives on 61–68% of ticks; mean total activation 779 per outburst (the paper's Fig. 7: 708 ± 230) and a mean wait of 22 ticks between outbursts (Fig. 5: 60 ± 55), a third of the paper's.",
            |c| model_one(c, 7.0, 0.82, Some(30), true, 0.04),
        ),
        preset(
            "cv-run-3-salami",
            "Run 3 — salami tactics",
            SOURCE,
            "Runs 3 and 4's inputs (vision 7, L = 0.9, terms that never end, cops 0.074) with C/A rounded down; from t = 77 legitimacy falls 0.01 a tick until it reaches 0.2 at t = 147. The paper (Fig. 9): no spike of actives, and a jail that fills smoothly, because each new rebel is picked off alone. Measured (seeds 1–20, paired with Run 4): 17 seeds stay under 50 actives while the jail fills (322–910 jailed by t = 300); 3 explode. Open Salami tactics vs one jump in the presets menu to run both.",
            |c| {
                model_one(c, 7.0, 0.9, None, true, 0.074);
                c.ramps = vec![Ramp {
                    path: "legitimacy".into(),
                    start: 77,
                    end: 147,
                    to: 0.2,
                }];
            },
        ),
        preset(
            "cv-run-4-one-jump",
            "Run 4 — one jump",
            SOURCE,
            "Runs 3 and 4's inputs with C/A rounded down; legitimacy drops from 0.9 to 0.7 in one step at t = 77. The paper (Fig. 10): an explosion of actives, and a jail that ends larger than Run 3's although legitimacy fell far less. Measured (seeds 1–20, paired with Run 3): the jump's peak beats salami tactics' in 17 seeds and passes 50 actives in 9 (salami: 3), but its jail ends larger in only 7 — the explosion reproduces about half the time, the larger jail does not.",
            |c| {
                model_one(c, 7.0, 0.9, None, true, 0.074);
                c.schedule = vec![at(77, "legitimacy", json!(0.7))];
            },
        ),
        preset(
            "cv-run-5-cop-reductions",
            "Run 5 — cop reductions",
            SOURCE,
            "Table 2's Run 5 (vision 7, L = 0.8, terms that never end, cops 0.074) with C/A rounded down; from t = 50 the cops are walked down in a straight line to none at t = 550. The paper (Fig. 11): unlike falling legitimacy, a marginal cut in cops tips the society into rebellion. Measured (seeds 1–20): every run tips — a peak of 64–345 actives (mean 194) at a mean t = 177, with about 88 of the 118 cops left, and most rebels jailed for good. With the paper's stated rule no run tips (peaks of 14–33).",
            |c| {
                model_one(c, 7.0, 0.8, None, true, 0.074);
                c.ramps = vec![Ramp {
                    path: "cop_density".into(),
                    start: 50,
                    end: 550,
                    to: 0.0,
                }];
            },
        ),
        preset(
            "cv-run-6-coexistence",
            "Run 6 — peaceful coexistence",
            SOURCE,
            "Table 2's Run 6: Model II (Blue and Green; going active means killing a member of the other group), vision 1.7, L = 0.9, no cops, cloning 0.05, death ages up to 200. The paper (Fig. 12): peaceful coexistence. At L = 0.9 grievance never exceeds the 0.1 threshold, so nobody ever goes active: no killing in 20 of 20 seeds.",
            |c| model_two(c, 0.9, 0.0),
        ),
        preset(
            "cv-run-7-cleansing",
            "Run 7 — ethnic cleansing",
            SOURCE,
            "Table 2's Run 7: Model II, L = 0.8, no cops, stopping once a group is gone. The paper (Fig. 13): local ethnic cleansing, and genocide in each of 30 runs with a random victor. Measured (seeds 1–20): genocide in every seed at t = 30–190 (mean 94); Blue survived 12 times, Green 8.",
            |c| {
                model_two(c, 0.8, 0.0);
                c.stop_at_extinction = true;
            },
        ),
        preset(
            "cv-run-8-nasty-regime",
            "Run 8 — cops from the start",
            SOURCE,
            "Table 2's Run 8: Model II, L = 0.8, cops 0.04 from the start. The paper's text says this gives 'a stable, but nasty, regime' in which the cops keep both groups alive; its own Fig. 15 shows rapid genocide at every cop density. Measured (seeds 1–20): one group is gone in every seed by t = 75–882 (mean 180) — the stable regime does not reproduce.",
            |c| model_two(c, 0.8, 0.04),
        ),
        preset(
            "cv-safe-havens",
            "Peacekeepers at t = 50",
            SOURCE,
            "Run 7 with peacekeepers deployed at t = 50 on random empty sites, stopping once a group is gone. The paper (Fig. 14) says this typically produces safe havens but gives no density; 0.04 is the density it calls high. Measured (seeds 1–20): genocide in every seed by t = 30–377 (mean 131). Denser forces only delay it: at 0.1, 0.2 and 0.3 (seeds 1–5) a group is still gone by t = 76–1475.",
            |c| {
                model_two(c, 0.8, 0.0);
                c.stop_at_extinction = true;
                c.schedule = vec![at(50, "cop_density", json!(0.04))];
            },
        ),
        preset(
            "cv-netlogo",
            "NetLogo Rebellion",
            "Wilensky 2004, NetLogo Rebellion",
            "NetLogo Rebellion's defaults (70% agents, 4% cops, vision 7, L = 0.82, J_max = 30) with all five of its departures from the paper: C/A rounded down, an active agent counted twice, the arresting cop stepping onto the arrest, jailed agents keeping their site, and terms of 0 to J_max − 1 ticks. Measured (seeds 1–20, 3000 ticks): 93–109 outbursts, mean total activation 1107, mean wait 23 ticks.",
            |c| c.quirks = Quirks::ALL,
        ),
    ]
}
