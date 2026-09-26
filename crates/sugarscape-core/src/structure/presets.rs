//! Table 2's structures and the paper's two readings of its own method.

use super::config::{NoiseOn, Start, Structure, StructureConfig};
use crate::model::ModelConfig;
use crate::presets::ModelPreset;

const CRA: &str = "Cohen, Riolo & Axelrod 2001";

fn preset(
    id: &'static str,
    name: &'static str,
    description: &'static str,
    edit: impl FnOnce(&mut StructureConfig),
) -> ModelPreset {
    let mut c = StructureConfig {
        stop_at: 2500,
        ..StructureConfig::default()
    };
    edit(&mut c);
    ModelPreset {
        id,
        name,
        source: CRA,
        description,
        config: ModelConfig::Structure(c),
    }
}

fn frn(c: &mut StructureConfig) {
    c.structure = Structure::Frn;
}

pub fn presets() -> Vec<ModelPreset> {
    vec![
        preset("cra-rwr", "Random partners each period (RWR)", "Cohen, Riolo and Axelrod's population: 256 agents each period play four-move Prisoner's Dilemmas with four partners (a strategy: cooperate first with probability y, after a cooperation with p, after a defection with q), then copy their best-scoring partner if it did strictly better — misjudging 10 % of the time, with 10 % noise on each of y, p, q. Here partners are drawn afresh every period (RWR, Table 2 row 1). The left block is the agents colored by friendliness p; the right plane is p against q with the population's recent trail. The paper: high cooperation reached in 0.30 of runs and held 1.5 % of the time after, a mean payoff of 1.091. Measured (30 seeds, 2500 periods, high = 2.3): a mean payoff of 1.089; reached in 0.43 of runs, held 2.0 % of the time.", |_| {}),
        preset("cra-2dk", "Torus neighbors (2DK)", "The same agents on a 16 × 16 torus, each playing its four NEWS neighbors — every pair twice a period — for the whole run (2DK, row 2). The block is the torus itself. The paper: every run reaches high cooperation and holds it 99.7 % of the time, a mean payoff of 2.557. Measured (30 seeds, 2500 periods, high = 2.3): 2.553; every run; held 99.8 %.", |c| {
            c.structure = Structure::Torus
        }),
        preset(
            "cra-frne",
            "Fixed random neighbors, symmetric (FRNE)",
            "Fixed random neighbors, four each and symmetric (FRNE, row 3): the torus's fixity without its clustering. The block's positions mean nothing here. The paper: 2.575, every run, held 99.5 % — and (note 5) better than 2DK. Measured (30 seeds, 2500 periods, high = 2.3): 2.574; every run; held 99.7 %; above 2DK's 2.553, as the paper says.",
            |c| c.structure = Structure::Frne,
        ),
        preset("cra-frn", "Fixed random neighbors (FRN)", "Fixed random neighbors, drawn once with replacement, one-way (FRN, row 4): context preservation alone. The paper: 2.480, every run, held 94.2 %. Measured (30 seeds, 2500 periods, high = 2.3): 2.478; every run; held 94.0 %. In the crucial region (population p 0.30–0.35, q 0.05–0.10) p rises 0.051 a period (the paper: 0.052) and an agent's partners' p tracks its own (slope 0.18; the paper 0.158).", frn),
        preset(
            "cra-ffr-01",
            "Fixed, 10 % substituted (FFR-0.1)",
            "FRN with each fixed partner replaced, for the period, by a random one with probability 0.1 (FFR-0.1, row 5). The paper: 2.385, held 84.4 %. Measured (30 seeds, 2500 periods, high = 2.3): 2.405; every run; held 84.3 %.",
            |c| {
                frn(c);
                c.substitution = 0.1;
            },
        ),
        preset(
            "cra-ffr-03",
            "Fixed, 30 % substituted (FFR-0.3)",
            "FFR-0.3 (row 6): 'around a parameter value of 0.3 the dynamics shift'; the paper calls it bi-stable. The paper: 2.100, held 40.2 %. Measured (30 seeds, 2500 periods, high = 2.3): 2.036; every run; held 37.6 %; in 25 of 30 runs stretches of at least 50 periods both high and low.",
            |c| {
                frn(c);
                c.substitution = 0.3;
            },
        ),
        preset(
            "cra-ffr-05",
            "Fixed, 50 % substituted (FFR-0.5)",
            "FFR-0.5 (row 7): 'at levels of 0.5 and above, it collapses'. The paper: reached in 0.93 of runs, 1.257, held 6.1 %. Measured (30 seeds, 2500 periods, high = 2.3): reached in 0.97, 1.325, held 7.3 %.",
            |c| {
                frn(c);
                c.substitution = 0.5;
            },
        ),
        preset("cra-random-start", "FRN from a random start", "FRN from random strategies (§3.1's 'initialized randomly') instead of the Appendix's even spread. Measured (30 seeds, 2500 periods, high = 2.3): 2.473 and held 93.8 % against 2.478 and 94.0 % from the even spread — the paper's two descriptions of its start make no difference.", |c| {
            frn(c);
            c.start = Start::Random;
        }),
        preset("cra-copy-noise", "FRN, noise only on copying", "FRN with noise only on agents that copy (§2's 'errors in the actual copying process') instead of on every agent every period (the Appendix). Measured (30 seeds, 2500 periods, high = 2.3): 2.530 and held 98.4 % against 2.478 and 94.0 % — less noise, more cooperation, overshooting Table 2; the Appendix's reading is the one that matches Table 2 more closely.", |c| {
            frn(c);
            c.noise_on = NoiseOn::Copy;
        }),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_set_what_they_say() {
        let got: Vec<(&str, StructureConfig)> = presets()
            .into_iter()
            .map(|p| match p.config {
                ModelConfig::Structure(c) => (p.id, c),
                _ => panic!("{} is not a social-structure preset", p.id),
            })
            .collect();
        let find = |id| got.iter().find(|(i, _)| *i == id).unwrap().1.clone();
        assert_eq!(find("cra-rwr").structure, Structure::Rwr);
        assert_eq!(find("cra-2dk").structure, Structure::Torus);
        let f = find("cra-ffr-03");
        assert_eq!((f.structure, f.substitution), (Structure::Frn, 0.3));
        assert_eq!(find("cra-random-start").start, Start::Random);
        assert_eq!(find("cra-copy-noise").noise_on, NoiseOn::Copy);
        assert!(got
            .iter()
            .all(|(_, c)| c.stop_at == 2500 && c.validate().is_ok()));
    }
}
