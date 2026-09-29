//! Balinese Water Temples' parameters: Lansing and Kremer's (1993) subaks on
//! the Oos and Petanu, Janssen's (2007) analyses of their model, and every
//! detail the texts leave open — and every place Janssen's code departs from
//! his text — as a named switch.

use serde::{Deserialize, Serialize};

use crate::config::FieldError;
use crate::schema::{Apply, Param};

/// The irrigation network.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Watershed {
    /// The Oos and Petanu rivers: 172 subaks, 12 dams.
    Bali,
    /// Janssen's two-node model (§4): an upstream and a downstream subak.
    TwoNode,
}

/// The starting plans.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Plans {
    /// Any of the 21 plans, from any month.
    Random,
    /// LK's kerta masa: six-month then four-month traditional rice (plan 6).
    Traditional,
    /// Two high-yielding crops a year (plan 1; LK's high-yielding runs added
    /// a vegetable crop, which the 21 plans drop).
    Hyv,
    /// One random plan per masceti temple.
    Temples,
    /// Janssen's hill-climbing search for the plan of each group at `level`.
    Search,
}

/// Each year's decision.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Decision {
    /// LK: copy the best neighbor's plan and start, if strictly better.
    Imitate,
    /// Janssen's eq. 4, with innovation below the mean.
    Generalized,
    /// Janssen §5: plant a three-month crop when water and pests allow.
    Adaptive,
    /// Plans never change.
    Fixed,
}

/// Rainfall.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rain {
    Low,
    Middle,
    High,
    /// A scenario drawn each year: low 25 %, middle 50 %, high 25 % (Janssen).
    Random,
}

/// How water passes between dams.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Routing {
    /// Down the dam network, upstream first (both texts).
    Network,
    /// Janssen's code: one random dam a month balances its own water, with no
    /// inflow from upstream; the others keep their last water stress.
    JanssenCode,
}

/// How the subak–dam file's columns are read.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DamColumns {
    /// As Janssen's code reads them: (return, source).
    Code,
    /// Swapped: the first is the upstream dam in 93 of 95 cases.
    Physical,
}

/// The pest equation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PestForm {
    /// LK's "shortcut": p' = g·(p + ½·flux) + ½·flux.
    Shortcut,
    /// The diffusion Janssen expected: p' = g·p + flux.
    Diffusion,
}

/// Lansing and Kremer's perturbation (Fig. 11).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Perturb {
    pub enabled: bool,
    /// The first perturbed year.
    pub at: u32,
    /// Pest growth and dispersal from then.
    pub growth: f64,
    pub dispersal: f64,
    /// Pest damage: sensitivity × this.
    pub damage: f64,
    /// Rain × this.
    pub rain: f64,
}

impl Default for Perturb {
    /// LK: "increasing the pest growth, dispersal, and damage rates.
    /// Simultaneously, rainfall was decreased to 80% of normal" — how much
    /// the pests rose is unstated: here the top of their ranges, and damage × 1.5.
    fn default() -> Self {
        Perturb {
            enabled: false,
            at: 21,
            growth: 2.4,
            dispersal: 0.45,
            damage: 1.5,
            rain: 0.8,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BaliConfig {
    pub watershed: Watershed,
    pub plans: Plans,
    /// Lansing and Kremer's coordination levels, for `Plans::Search`: 1, 2, 7,
    /// 14, 28 or 172. Level 2 is our two rivers (Janssen: highlands and
    /// lowlands, not identifiable in the data); level 28, the mascetis split
    /// by the data's second temple column, has only 22 non-empty groups.
    pub level: u32,
    pub decision: Decision,
    /// g: pest growth with rice in the field (0.1 when fallow).
    pub growth: f64,
    /// d: pest dispersal.
    pub dispersal: f64,
    pub rain: Rain,
    pub rain_scale: f64,
    pub perturb: Perturb,
    pub routing: Routing,
    pub dam_columns: DamColumns,
    pub pest_form: PestForm,
    /// Pests back to 0.01 each year (Janssen's code).
    pub pest_reset: bool,
    /// Eq. 4's γp and γw, and innovation ρ.
    pub gamma_p: f64,
    pub gamma_w: f64,
    pub innovation: f64,
    /// Adaptive thresholds: water expected at the source dam (m/day per
    /// hectare it serves) and pests per subak in the neighborhood.
    pub m_w: f64,
    pub m_p: f64,
    /// Janssen's pₑ and pₙ: each pest link removed, and each pair of subaks
    /// sharing a source or return dam linked, with these probabilities.
    pub remove_links: f64,
    pub add_links: f64,
    /// Two nodes: rain units a month, and periods a year (2 or 12).
    pub node_rain: f64,
    pub node_periods: u32,
    /// The first scored year (Janssen: the last five of ten).
    pub score_from: u32,
    /// Stop after this many years (0: never).
    pub stop_at: u32,
}

impl Default for BaliConfig {
    /// LK's coadaptation run: random plans, imitation, g 2.2, d 0.3, middle
    /// rain, 30 years.
    fn default() -> Self {
        BaliConfig {
            watershed: Watershed::Bali,
            plans: Plans::Random,
            level: 14,
            decision: Decision::Imitate,
            growth: 2.2,
            dispersal: 0.3,
            rain: Rain::Middle,
            rain_scale: 1.0,
            perturb: Perturb::default(),
            routing: Routing::Network,
            dam_columns: DamColumns::Code,
            pest_form: PestForm::Shortcut,
            pest_reset: true,
            gamma_p: 0.4,
            gamma_w: 0.4,
            innovation: 0.04,
            m_w: 0.05,
            m_p: 0.02,
            remove_links: 0.0,
            add_links: 0.0,
            node_rain: 2.0,
            node_periods: 12,
            score_from: 6,
            stop_at: 30,
        }
    }
}

/// The coordination levels, by Lansing and Kremer's names (level 28 has 22
/// groups in the data).
pub const LEVELS: [u32; 6] = [1, 2, 7, 14, 28, 172];

impl BaliConfig {
    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut e = Vec::new();
        let mut check = |ok: bool, field: &str, message: &str| {
            if !ok {
                e.push(FieldError::new(field, message));
            }
        };
        let unit = |v: f64| (0.0..=1.0).contains(&v);
        check(
            LEVELS.contains(&self.level),
            "level",
            "must be 1, 2, 7, 14, 28 or 172",
        );
        check(
            (0.0..=20.0).contains(&self.growth),
            "growth",
            "must be between 0 and 20",
        );
        check(
            (0.0..=2.0).contains(&self.dispersal),
            "dispersal",
            "must be between 0 and 2",
        );
        check(
            (0.0..=3.0).contains(&self.rain_scale),
            "rain_scale",
            "must be between 0 and 3",
        );
        let p = &self.perturb;
        check(
            p.at >= 1
                && (0.0..=20.0).contains(&p.growth)
                && (0.0..=2.0).contains(&p.dispersal)
                && (0.0..=10.0).contains(&p.damage)
                && (0.0..=3.0).contains(&p.rain),
            "perturb",
            "needs a year from 1, growth 0–20, dispersal 0–2, damage 0–10 and rain 0–3",
        );
        check(
            (0.0..=100.0).contains(&self.gamma_p),
            "gamma_p",
            "must be between 0 and 100",
        );
        check(
            (0.0..=100.0).contains(&self.gamma_w),
            "gamma_w",
            "must be between 0 and 100",
        );
        check(
            unit(self.innovation),
            "innovation",
            "must be between 0 and 1",
        );
        check(
            (0.0..=10.0).contains(&self.m_w),
            "m_w",
            "must be between 0 and 10",
        );
        check(
            (0.0..=100.0).contains(&self.m_p),
            "m_p",
            "must be between 0 and 100",
        );
        check(
            unit(self.remove_links),
            "remove_links",
            "must be between 0 and 1",
        );
        check(unit(self.add_links), "add_links", "must be between 0 and 1");
        check(
            (0.0..=10.0).contains(&self.node_rain),
            "node_rain",
            "must be between 0 and 10",
        );
        check(
            self.node_periods == 2 || self.node_periods == 12,
            "node_periods",
            "must be 2 or 12",
        );
        check(self.score_from >= 1, "score_from", "must be at least 1");
        check(self.stop_at <= 100_000, "stop_at", "must be at most 100000");
        if e.is_empty() {
            Ok(())
        } else {
            Err(e)
        }
    }

    /// Fields that change only on reset and differ in `next`.
    pub(crate) fn structural_changes(&self, next: &BaliConfig) -> Vec<FieldError> {
        let mut out = Vec::new();
        for (field, same) in [
            ("watershed", self.watershed == next.watershed),
            ("plans", self.plans == next.plans),
            ("level", self.level == next.level),
            ("dam_columns", self.dam_columns == next.dam_columns),
            ("remove_links", self.remove_links == next.remove_links),
            ("add_links", self.add_links == next.add_links),
            ("node_rain", self.node_rain == next.node_rain),
            ("node_periods", self.node_periods == next.node_periods),
        ] {
            if !same {
                out.push(FieldError::new(field, "changes only on reset"));
            }
        }
        out
    }
}

/// The Rules panel's fields.
pub fn schema() -> Vec<Param> {
    use Apply::{Live, Reset};
    vec![
        Param::choice(
            "Watershed",
            "watershed",
            "Watershed",
            &[
                ("bali", "The Oos and Petanu (172 subaks)"),
                ("two_node", "Two subaks (Janssen)"),
            ],
            Reset,
        ),
        Param::number("Watershed", "node_rain", "Rain units a month", (0.0, 4.0, 0.1), Reset)
            .shown_if("watershed", "two_node"),
        Param::integer("Watershed", "node_periods", "Periods a year", (2, 12), Reset)
            .shown_if("watershed", "two_node")
            .with_help("2 or 12 (Janssen's Fig. 5; Figs. 6–8)."),
        Param::choice(
            "Plans",
            "plans",
            "Starting plans",
            &[
                ("random", "Random"),
                ("traditional", "Traditional (kerta masa)"),
                ("hyv", "Two high-yielding crops"),
                ("temples", "One per temple"),
                ("search", "Found by search (Janssen)"),
            ],
            Reset,
        ),
        Param::integer("Plans", "level", "Groups sharing a plan", (1, 172), Reset)
            .shown_if("plans", "search")
            .with_help("1 (the watershed), 2 (the two rivers: Janssen's highlands and lowlands are not in the data), 7 (pairs of temples), 14 (the temples), 28 (the temples split by the data's second temple column: only 22 groups) or 172."),
        Param::choice(
            "Decisions",
            "decision",
            "Each year, subaks",
            &[
                ("imitate", "Copy their best neighbor (Lansing & Kremer)"),
                ("generalized", "Copy by network distance, or innovate (Janssen)"),
                ("adaptive", "Plant when water and pests allow (Janssen)"),
                ("fixed", "Keep their plans"),
            ],
            Live,
        ),
        Param::number("Decisions", "gamma_p", "γp (pest distance)", (0.0, 5.0, 0.05), Live)
            .shown_if("decision", "generalized"),
        Param::number("Decisions", "gamma_w", "γw (water distance)", (0.0, 5.0, 0.05), Live)
            .shown_if("decision", "generalized"),
        Param::number("Decisions", "innovation", "Innovation (ρ)", (0.0, 1.0, 0.01), Live)
            .shown_if("decision", "generalized"),
        Param::number("Decisions", "m_w", "Water to plant (m/day/ha)", (0.0, 1.0, 0.005), Live)
            .shown_if("decision", "adaptive"),
        Param::number("Decisions", "m_p", "Pests to plant under", (0.0, 1.0, 0.005), Live)
            .shown_if("decision", "adaptive"),
        Param::number("Pests", "growth", "Pest growth (g)", (0.0, 4.0, 0.01), Live)
            .with_help("With rice in the field; 0.1 when fallow. Lansing & Kremer: 2–2.4."),
        Param::number("Pests", "dispersal", "Pest dispersal (d)", (0.0, 1.0, 0.01), Live)
            .with_help("Lansing & Kremer: 0.18–0.45."),
        Param::choice(
            "Pests",
            "pest_form",
            "Pest equation",
            &[
                ("shortcut", "Lansing & Kremer's shortcut"),
                ("diffusion", "Diffusion (Janssen's expectation)"),
            ],
            Live,
        ),
        Param::bool("Pests", "pest_reset", "Pests reset each year", Live)
            .with_help("Janssen's code; without it, he says, harvests lock low."),
        Param::bool("Pests", "perturb.enabled", "Pests and drought strike (Fig. 11)", Live),
        Param::integer("Pests", "perturb.at", "From year", (1, 100_000), Live)
            .shown_if("perturb.enabled", "true"),
        Param::choice(
            "Water",
            "rain",
            "Rain",
            &[
                ("middle", "Middle"),
                ("low", "Low"),
                ("high", "High"),
                ("random", "Random year by year"),
            ],
            Live,
        ),
        Param::number("Water", "rain_scale", "Rain ×", (0.0, 2.0, 0.05), Live),
        Param::choice(
            "Water",
            "routing",
            "Water flows",
            &[
                ("network", "Down the dam network (the texts)"),
                ("janssen_code", "One random dam a month (Janssen's code)"),
            ],
            Live,
        ),
        Param::choice(
            "Water",
            "dam_columns",
            "Dam columns",
            &[
                ("code", "As Janssen's code reads them"),
                ("physical", "Swapped (upstream as source)"),
            ],
            Reset,
        ),
        Param::number("Network", "remove_links", "Remove pest links (pₑ)", (0.0, 1.0, 0.05), Reset),
        Param::number("Network", "add_links", "Add pest links (pₙ)", (0.0, 1.0, 0.01), Reset),
        Param::integer("Stopping", "score_from", "Score from year", (1, 100_000), Live)
            .with_help("Janssen scores the last five of ten years."),
        Param::integer("Stopping", "stop_at", "Stop after year", (0, 100_000), Live)
            .with_help("0: never."),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ModelConfig, ModelWorld};

    #[test]
    fn defaults_are_lansing_and_kremers_run() {
        let c = BaliConfig::default();
        assert_eq!(
            (c.growth, c.dispersal, c.level, c.stop_at),
            (2.2, 0.3, 14, 30)
        );
        assert_eq!(
            (c.plans, c.decision, c.routing),
            (Plans::Random, Decision::Imitate, Routing::Network)
        );
        assert!(c.validate().is_ok());
    }

    #[test]
    fn validation_names_fields() {
        let bad = BaliConfig {
            level: 3,
            growth: -1.0,
            dispersal: 3.0,
            rain_scale: 4.0,
            perturb: Perturb {
                at: 0,
                ..Perturb::default()
            },
            innovation: 2.0,
            remove_links: 1.5,
            node_periods: 3,
            score_from: 0,
            stop_at: 200_000,
            ..BaliConfig::default()
        };
        let fields: Vec<String> = bad
            .validate()
            .unwrap_err()
            .into_iter()
            .map(|e| e.field)
            .collect();
        assert_eq!(
            fields,
            [
                "level",
                "growth",
                "dispersal",
                "rain_scale",
                "perturb",
                "innovation",
                "remove_links",
                "node_periods",
                "score_from",
                "stop_at"
            ]
        );
    }

    #[test]
    fn the_plans_and_network_change_only_on_reset() {
        let next = BaliConfig {
            plans: Plans::Traditional,
            growth: 2.4,
            ..BaliConfig::default()
        };
        let changes = BaliConfig::default().structural_changes(&next);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].field, "plans");
    }

    #[test]
    fn schema_paths_exist_and_match_what_set_config_allows() {
        let config = ModelConfig::Bali(BaliConfig::default());
        crate::schema::check_schema(&schema(), &config, || {
            ModelWorld::new(config.clone(), 1).unwrap()
        });
    }
}
