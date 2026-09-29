//! Lansing and Kremer's coadaptation runs and Janssen's analyses of them.

use super::config::{BaliConfig, Decision, Perturb, Plans, Routing, Watershed};
use crate::model::ModelConfig;
use crate::presets::ModelPreset;

const LK: &str = "Lansing & Kremer 1993, Am. Anthropol. 95: 97";
const J: &str = "Janssen 2007, Agric. Syst. 93: 170";

fn preset(
    id: &'static str,
    name: &'static str,
    source: &'static str,
    description: &'static str,
    edit: impl FnOnce(&mut BaliConfig),
) -> ModelPreset {
    let mut c = BaliConfig::default();
    edit(&mut c);
    ModelPreset {
        id,
        name,
        source,
        description,
        config: ModelConfig::Bali(c),
    }
}

pub fn presets() -> Vec<ModelPreset> {
    vec![
        preset("lk-random", "Imitation from random plans", LK, "Lansing and Kremer's coadaptation on Janssen's data for the Oos and Petanu: 172 subaks, 12 dams, middle rain, pests growing at g 2.2 and spreading at d 0.3 along the mapped pest links. Every subak starts with a random one of Janssen's 21 plans from a random month; at each year's end each copies its best-harvesting neighbor if that one did strictly better, and pests reset. Measured (10 seeds × 30 years): 10.9 → 19.4 t/ha/yr by year 8 and 20.4 by year 30; subaks changing 97 → 7 → 2 (the paper: 'all but 20'). The final patches match the 14 masceti temples with an adjusted Rand index of 0.37 — but the pest network's own components already match them at 0.33.", |_| {}),
        preset(
            "lk-random-fixed",
            "Random plans, fixed",
            LK,
            "The control for lk-random: the same random plans and start months (the same seed draws them), never changed. Measured (10 seeds × 30 years): 10.8 t/ha/yr every year, against 20.4 when the subaks imitate.",
            |c| c.decision = Decision::Fixed,
        ),
        preset("lk-traditional", "Imitation, traditional rice", LK, "Every subak starts with the traditional pattern (six-month rice, fallow, four-month rice, fallow) from a random month, and imitates. Measured (10 seeds × 30 years): 5.0 → 8.1 t/ha/yr (Table 1: 4.9 → 8.57), as the subaks line up their fallows.", |c| {
            c.plans = Plans::Traditional
        }),
        preset("lk-hyv", "Imitation, high-yielding rice", LK, "Every subak starts with two crops of high-yielding rice a year from a random month (Lansing and Kremer's runs add a vegetable crop; Janssen's 21 plans drop it), and imitates. Measured (10 seeds × 30 years): 16.9 → 18.2 t/ha/yr (Table 1: 15.91 → 18.08); under one subak still changing by year 20 (the paper: 20).", |c| c.plans = Plans::Hyv),
        preset("lk-perturbed", "Pests and drought in year 21", LK, "Fig. 11: high-yielding plans imitate for 20 years; from year 21 pests grow at 2.4, spread at 0.45 and do 1.5 times the damage, and rain falls to 80 % (the paper's magnitudes are not given; these are ours). Measured (10 seeds × 40 years): 18.1 → 16.5 t/ha/yr in year 21, and no recovery (the paper: 15.3, recovering to 15.8 within seven years) — the imitating subaks have almost stopped changing, and nothing better is in reach.", |c| {
            c.plans = Plans::Hyv;
            c.perturb = Perturb { enabled: true, ..Perturb::default() };
            c.stop_at = 40;
        }),
        preset("lk-stressed", "Pests and drought from the start", LK, "The same pests and drought from the first year. Measured (10 seeds × 40 years): 12.9 → 16.2 t/ha/yr by year 5 and 16.5 by year 30 (Table 1, low rain and high pests: 13.67 → 17.66).", |c| {
            c.plans = Plans::Hyv;
            c.perturb = Perturb { enabled: true, at: 1, ..Perturb::default() };
            c.stop_at = 40;
        }),
        preset("lk-temples", "Plans fixed by temple", LK, "No imitation: each masceti temple's subaks share one random plan and start month, fixed. Measured (10 seeds): 13.0 t/ha/yr, against 20.4 when subaks imitate from random plans — a random plan per temple is not the temples' plan.", |c| {
            c.plans = Plans::Temples;
            c.decision = Decision::Fixed;
        }),
        preset("janssen-code", "As Janssen's code routes water", J, "Lansing and Kremer's run with water as Janssen's code routes it: each month one random dam balances its own water, with no inflow from upstream, and the others keep their last value (all starting dry, so the first year yields 4.9). Measured (10 seeds × 30 years): 20.1 t/ha/yr by year 30, within 2 % of the network routing's — water hardly binds on these rivers.", |c| {
            c.routing = Routing::JanssenCode;
        }),
        preset("janssen-levels-14", "Best plan per temple", J, "Janssen's optimization at the temple scale: the subaks of each masceti share one plan and start month, found by hill-climbing over all 252 (every group in turn tries every option, up to five passes), scoring year 2 of a two-year run; then fixed for ten years. Measured (3 seeds): 27.1 t/ha/yr, the best of the six levels by 0.1 % — one plan for the whole watershed gives 27.05 (Janssen's Fig. 1: ≈ 17.5 at one group rising to 22.8 at 172).", |c| {
            c.plans = Plans::Search;
            c.decision = Decision::Fixed;
            c.stop_at = 10;
        }),
        preset("janssen-two-node", "Two subaks, one river", J, "Janssen's §4: an upstream and a downstream subak with two units of rain a month, pests growing at 2.2 and spreading at 0.3 by his eq. 2; each plants one of five three-month-crop patterns from any month, the best pair found by trying all 3 600. Measured: 4.51 crops' worth a year — above g ≈ 2.14 (∛10) pests outgrow a month's fallow and six crops are no longer possible (5.44 at g 2.1).", |c| {
            c.watershed = Watershed::TwoNode;
        }),
        preset("janssen-generalized", "Imitation by network distance", J, "Janssen's eq. 4: a subak copies the best-harvesting subak j anywhere whose harvest beats its own by the factor 1 + min(γp χp², γw χw²), χ counting hops in the pest and water networks (γ 0.4 each); below the average harvest it tries a random plan with probability 0.04. Measured (10 seeds × 30 years): 25.4 t/ha/yr, above neighbor imitation's 20.4, with fewer strategies (18 against 43) and less inequality.", |c| {
            c.decision = Decision::Generalized;
        }),
        preset("janssen-adaptive", "Adaptive subaks", J, "Janssen's adaptive subaks: no plans; each month a fallow subak plants three-month rice if its source dam's water per hectare served exceeds 0.05 m/day (our reading of mw) and its neighborhood's mean pests are below 0.02. Measured: 25.95 t/ha/yr from the second year on, the same on every seed (middle rain is fixed and the rule has no chance in it), with the highest inequality of any rule (spread 10.3).", |c| {
            c.decision = Decision::Adaptive;
        }),
        preset("janssen-fewer-links", "Imitation with pest links removed", J, "Lansing and Kremer's imitation with each pest link removed with probability 0.5 (Janssen's pₑ). Measured (10 seeds × 30 years): 16.2 t/ha/yr — imitation stops by year 8 with 81 strategies in use, as isolated subaks have no one to copy.", |c| {
            c.remove_links = 0.5;
        }),
    ]
}
