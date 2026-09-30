//! Axtell's base case, its readings, a preset for each §4 variation at its
//! most striking setting, and the 2013 parameterization.

use super::config::{
    Activation, FirmsConfig, Network, OthersEffort, Pay, Preferences, RandomBehavior,
};
use crate::model::ModelConfig;
use crate::presets::ModelPreset;

const A99: &str = "Axtell 1999, Brookings CSED WP 3";
const A13: &str = "Axtell 2013, Endogenous Dynamics of Firms and Labor";

fn preset(
    id: &'static str,
    name: &'static str,
    source: &'static str,
    description: &'static str,
    edit: impl FnOnce(&mut FirmsConfig),
) -> ModelPreset {
    let mut c = FirmsConfig::default();
    edit(&mut c);
    ModelPreset {
        id,
        name,
        source,
        description,
        config: ModelConfig::Firms(c),
    }
}

pub fn presets() -> Vec<ModelPreset> {
    vec![
        preset("firms-base", "Base case", A99, "Axtell's base case (his Table 2): 1 000 agents with preferences θ ~ U[0, 1] between income and leisure, each with 2 random friends; a firm makes a·E + b·E² (a = b = 1) from its members' total effort and shares it equally; 1 000 random activations a period, each agent weighing its own firm, starting alone and its friends' firms at its best effort, others' effort read from last period's output (the text's reading); everyone starts alone. Measured (5 seeds × 5 000 periods, after 500): about 435 firms, mean size 2.3, the largest firm reaching 62–139, µ 2.29–2.79 by the paper's OLS (1.38–1.50 by maximum likelihood) against Axtell's 1.28, and a mean firm lifetime of 3.9 periods against his 23.4.", |_| {}),
        preset("firms-live", "Base case, others' effort live", A99, "The base case with others' effort read live (their current efforts), not from last period's output. Measured (5 seeds × 5 000 periods, after 500): about 374 firms, the largest reaching 195–318 (Axtell's typical run: about 205), µ 1.91–2.26 (maximum likelihood 1.25–1.38), lifetime 3.2 — the closest reading to the paper, still far from µ 1.28 and lifetimes of 23.4.", |c| c.others_effort = OthersEffort::Live),
        preset("firms-uniform", "Base case, uniform activation", A99, "The base case with each agent activated once a period, in random order (Axtell: 'essentially no difference'). Measured (5 seeds × 5 000 periods, after 500): about 466 firms, the largest 59–153, µ 2.35–2.79, lifetime 2.7 — like the base case.", |c| c.activation = Activation::Uniform),
        preset("firms-beta-17", "Weaker increasing returns (β 1.7)", A99, "The base case with weaker increasing returns, β = 1.7 (Table 3: µ 2.06). Measured (5 seeds × 5 000 periods, after 500): about 540 firms, the largest only 25–32, µ 3.55–3.81 (maximum likelihood 1.85–1.91).", |c| c.beta = 1.7),
        preset("firms-beta-21", "Stronger increasing returns (β 2.1)", A99, "The base case with stronger increasing returns, β = 2.1 (Table 3: µ 0.95). Measured (5 seeds × 5 000 periods, after 500): about 376 firms, the largest 192–394, µ 1.83–2.32 (maximum likelihood 1.20–1.36) — larger firms, as Table 3's direction says.", |c| c.beta = 2.1),
        preset("firms-b-15", "Stronger increasing returns (b 1.5)", A99, "The base case with b = 1.5 (Table 4: µ 0.53). Measured (5 seeds × 5 000 periods, after 500): about 363 firms, the largest 127–324, µ 1.88–2.39, total output about 1 000.", |c| c.b = 1.5),
        preset("firms-b-random", "Each firm draws its b", A99, "Each firm draws its b from [0.5, 1.5] at founding (Table 4: µ 0.89, 'much more like b = 1.25'). Measured (5 seeds × 5 000 periods, after 500): about 347 firms, the largest 198–397, µ 1.68–2.10.", |c| {
            c.b = 0.5;
            c.b_max = 1.5;
        }),
        preset("firms-theta-075", "Everyone alike (θ 0.75)", A99, "Everyone alike: every θ = 0.75 (Table 5: µ 0.91). Measured (5 seeds × 5 000 periods, after 500): about 91 firms of mean size 11.5, but none beyond 40 — µ 1.10–1.30 by OLS on a distribution with no tail (maximum likelihood 0.50); lifetime 7.7; mean utility 1.05, well above the base case's 0.73.", |c| {
            c.preferences = Preferences::Fixed;
            c.theta = 0.75;
        }),
        preset("firms-friends-10", "Ten friends each", A99, "Ten friends each (Table 6: µ 0.99). Measured (5 seeds × 5 000 periods, after 500): about 259 firms of mean size 3.9, the largest only 49–61, µ 2.53–2.83 (maximum likelihood 0.99–1.11) — more mid-sized firms and fewer large ones.", |c| c.neighbors = 10),
        preset("firms-random-firms-10", "Ten random firms each time", A99, "Agents look at ten random firms each time instead of friends' firms (Table 7: µ 1.03). Measured (5 seeds × 5 000 periods, after 500): about 242 firms, the largest 49–57, µ 2.67–2.94, lifetime 7.3.", |c| {
            c.network = Network::RandomFirms;
            c.neighbors = 10;
        }),
        preset("firms-loyal-10", "Loyal agents (λ 10)", A99, "Loyal agents: each moves only after wanting to 11 times (λ = 10; Table 8: µ 0.77). Measured (5 seeds × 5 000 periods, after 500): about 307 firms, the largest 94–177, µ 1.83–2.44, and firms live 34.8 periods on average — loyalty is the one change that gives the paper's long lifetimes.", |c| c.loyalty = 10),
        preset("firms-sticky", "Sticky effort (±0.05)", A99, "Sticky effort (Table 9): each new effort within ±0.05 of the agent's current one — read as applying in any firm, joined or founded too. Measured (5 seeds × 5 000 periods, after 500): the whole population ends up in one firm at times in every seed (the largest 1 000), the largest firm making 59 % of all output; free riding sets in too slowly to stop joiners. Axtell reports a milder effect (µ 0.92 against 1.28). Restricted to the agent's own firm, sticky effort behaves like the base case with somewhat larger firms (the largest 176–189 over 3 seeds; the firms-sticky sweep).", |c| c.effort_window = 0.1),
        preset("firms-groping", "Groping for effort", A99, "Groping (Table 10): one random try at a new effort, kept if it does better; other firms weighed at the current effort. Measured (5 seeds × 5 000 periods, after 500): as with sticky effort, everyone joins one firm at times in every seed (the largest firm makes 78 % of output). Restricted to the agent's own firm, groping behaves like the base case (the firms-groping sweep).", |c| c.groping = true),
        preset("firms-seniority-5", "Seniority pay (5^−rank)", A99, "Seniority pay (Table 11): shares ∝ 5^−rank, the longest-serving member first, as the text says. Measured (5 seeds × 5 000 periods, after 500): firms barely form — the largest has 4 members, µ about 10 — since a joiner's share of a pair's output is 1/31. Axtell's µ 1.11 comes neither from this reading nor from paying the newest most (µ about 0.2; the firms-seniority sweep).", |c| {
            c.pay = Pay::Seniority;
            c.seniority_base = 5.0;
        }),
        preset("firms-base-pay-80", "Base pay at 80 % of singleton income", A99, "Base pay (Table 12): each agent is paid 80 % of its own singleton income, plus an equal share of what output exceeds the base pay (Axtell: µ 0.85). Measured (5 seeds × 5 000 periods, after 500): about 307 firms, the largest 183–267, µ 1.60–1.95, lifetime 14.7 — but effort collapses to 0.10 and total output to about 290, under two fifths of the base case's.", |c| {
            c.pay = Pay::Base;
            c.base_share = 0.8;
        }),
        preset("firms-hiring-100", "Hiring only the as-hardworking", A99, "A hiring standard of 100 %: a firm admits only agents whose θ is at least its longest-serving member's (Table 13, captioned 'target output': 'not well described by a power law'). Measured (5 seeds × 5 000 periods, after 500): about 719 firms, none beyond 20–37 members.", |c| c.hiring = 1.0),
        preset("firms-random-choices", "Random choices", A99, "§4.1's random behavior: each activated agent stays, moves or starts up at random (a random friend's firm), then chooses its best effort ('firms greater than 9 or 10 are rarely observed'). Measured (5 seeds × 5 000 periods, after 500): about 692 firms, the largest 19–23, µ 3.99–4.14.", |c| c.random_behavior = RandomBehavior::Choices),
        preset("firms-2013", "Axtell's 2013 parameterization", A13, "Axtell's 2013 parameterization at 10 000 agents: each firm draws a from [0, ½], b from [¾, 5/4] and β from [3/2, 2]; 2–6 friends each; 4 % of agents activated a period (his month). Measured (5 seeds × 2 000 periods, after 500): about 1 455 firms, µ 0.98–1.05 by OLS (0.88–0.90 by maximum likelihood) — Zipf's law, as the 2013 paper reports (α ≈ 1.06) — with firms living 77 periods on average; but the largest firm grows to 3 000–5 800 agents.", |c| {
            c.agents = 10_000;
            c.a = 0.0;
            c.a_max = 0.5;
            c.b = 0.75;
            c.b_max = 1.25;
            c.beta = 1.5;
            c.beta_max = 2.0;
            c.neighbors = 2;
            c.neighbors_max = 6;
            c.activation_rate = 0.04;
            c.stop_at = 2000;
        }),
    ]
}
