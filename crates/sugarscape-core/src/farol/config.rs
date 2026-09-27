//! The El Farol and Minority Game model's parameters: Arthur's (1994) bar with
//! a stated predictor library, Challet and Zhang's (1997) minority game with
//! its payoff and Darwinian variants, and Challet, Marsili and Ottino's
//! (2004) scoring, bias and random baseline, as named switches.

use serde::{Deserialize, Serialize};

use crate::config::FieldError;
use crate::schema::{Apply, Param};

/// Which game.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Game {
    /// Arthur: predictors of attendance numbers.
    ElFarol,
    /// Challet and Zhang: tables from the last M outcomes to a side.
    Minority,
}

/// How agents decide.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Behavior {
    /// By their best strategy or predictor.
    Inductive,
    /// Attending with probability capacity / N (CMO's zero-intelligence
    /// agents), or on a fair coin in the plain minority game.
    Random,
}

/// How El Farol's predictors are rated.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scoring {
    /// Arthur: the lowest decaying mean of |forecast − attendance|.
    Error,
    /// CMO: a point whenever the predictor's advice (go or stay) was right.
    Payoff,
}

/// What an agent does when its forecast is exactly the capacity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AtCapacity {
    /// Arthur: go only when expecting "fewer than 60".
    Stay,
    Go,
}

/// The minority game's payoff to each winner.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Payoff {
    /// A point per win.
    Step,
    /// N/x − 2 for x winners (CZ97 Fig. 4).
    Inverse,
}

/// How the inverse payoff is rounded.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rounding {
    /// CZ97: "these many (nearest integer values) points".
    Nearest,
    Exact,
}

/// Which history the minority game's strategies read.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Information {
    /// The real record of winning sides.
    True,
    /// A uniform random history each round.
    Random,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct MixedMemory {
    pub enabled: bool,
    pub min: u32,
    pub max: u32,
}

impl Default for MixedMemory {
    /// CZ97 Fig. 2's memories 1 to 10.
    fn default() -> Self {
        MixedMemory {
            enabled: false,
            min: 1,
            max: 10,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Evolution {
    pub enabled: bool,
    /// Rounds between replacements (CZ97: "after some time steps").
    pub every: u32,
    /// The chance the copy has one strategy redrawn.
    pub strategy_mutation: f64,
    /// The chance the copy's memory moves one up or down.
    pub memory_mutation: f64,
}

impl Default for Evolution {
    fn default() -> Self {
        Evolution {
            enabled: false,
            every: 100,
            strategy_mutation: 0.1,
            memory_mutation: 0.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct FarolConfig {
    pub game: Game,
    /// N.
    pub agents: u32,
    /// Arthur's k or CZ97's S.
    pub strategies: u32,
    pub behavior: Behavior,
    /// L; empty (null) is the game's own: 60 % of N at El Farol, (N − 1)/2
    /// in the minority game.
    pub capacity: Option<u32>,
    pub scoring: Scoring,
    /// λ in the error score s ← λs + (1 − λ)|forecast − A|.
    pub decay: f64,
    pub at_capacity: AtCapacity,
    /// Every agent holds the whole library.
    pub shared: bool,
    /// M.
    pub memory: u32,
    pub mixed_memory: MixedMemory,
    pub payoff: Payoff,
    pub rounding: Rounding,
    /// ā: the chance a strategy's entry says attend.
    pub bias: f64,
    pub information: Information,
    pub evolution: Evolution,
    /// Stop at this round (0: never).
    pub stop_at: u32,
}

impl Default for FarolConfig {
    /// Arthur's bar: 100 agents, 60 seats, 12 predictors each, rated by accuracy.
    fn default() -> Self {
        FarolConfig {
            game: Game::ElFarol,
            agents: 100,
            strategies: 12,
            behavior: Behavior::Inductive,
            capacity: None,
            scoring: Scoring::Error,
            decay: 0.9,
            at_capacity: AtCapacity::Stay,
            shared: false,
            memory: 3,
            mixed_memory: MixedMemory::default(),
            payoff: Payoff::Step,
            rounding: Rounding::Nearest,
            bias: 0.5,
            information: Information::True,
            evolution: Evolution::default(),
            stop_at: 0,
        }
    }
}

/// The library's size: El Farol's most predictors per agent.
pub const LIBRARY: u32 = 48;
/// The minority game's most strategies per agent, and longest memory.
pub const MAX_STRATEGIES: u32 = 16;
pub const MAX_MEMORY: u32 = 16;
/// The most strategy-table bits a world may hold (N · S · 2^M).
pub const BIT_BUDGET: u64 = 1 << 28;

impl FarolConfig {
    /// The longest memory any agent can have.
    pub fn top_memory(&self) -> u32 {
        if self.mixed_memory.enabled {
            self.mixed_memory.max
        } else {
            self.memory
        }
    }

    /// The longest memory the bit budget allows for N agents with S
    /// strategies (evolution never grows a memory past it).
    pub fn memory_ceiling(&self) -> u32 {
        let per = u64::from(self.agents) * u64::from(self.strategies.max(1));
        (1..=MAX_MEMORY)
            .rev()
            .find(|&m| per << m <= BIT_BUDGET)
            .unwrap_or(1)
    }

    /// L: the capacity set, or the game's own.
    pub fn capacity(&self) -> u32 {
        self.capacity.unwrap_or(match self.game {
            Game::ElFarol => (self.agents * 6 + 5) / 10,
            Game::Minority => self.agents.saturating_sub(1) / 2,
        })
    }

    /// The side a strategy entry names is compared with `capacity`: it is the
    /// plain minority game when capacity = (N − 1)/2.
    pub fn plain_minority(&self) -> bool {
        self.game == Game::Minority && 2 * self.capacity() + 1 == self.agents
    }

    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut e = Vec::new();
        let mut check = |ok: bool, field: &str, message: &str| {
            if !ok {
                e.push(FieldError::new(field, message));
            }
        };
        let unit = |v: f64| (0.0..=1.0).contains(&v);
        check(
            (3..=2001).contains(&self.agents),
            "agents",
            "must be between 3 and 2001",
        );
        let most = match self.game {
            Game::ElFarol => LIBRARY,
            Game::Minority => MAX_STRATEGIES,
        };
        check(
            (1..=most).contains(&self.strategies),
            "strategies",
            match self.game {
                Game::ElFarol => "must be between 1 and 48 (the library's size)",
                Game::Minority => "must be between 1 and 16",
            },
        );
        check(
            self.capacity() <= self.agents,
            "capacity",
            "must be at most the number of agents",
        );
        check(unit(self.decay), "decay", "must be between 0 and 1");
        check(unit(self.bias), "bias", "must be between 0 and 1");
        check(
            (1..=MAX_MEMORY).contains(&self.memory),
            "memory",
            "must be between 1 and 16",
        );
        let mm = &self.mixed_memory;
        check(
            !mm.enabled || (1 <= mm.min && mm.min <= mm.max && mm.max <= MAX_MEMORY),
            "mixed_memory",
            "needs 1 ≤ min ≤ max ≤ 16",
        );
        if self.game == Game::Minority {
            check(
                self.top_memory() <= self.memory_ceiling(),
                "memory",
                "too long for this many agents and strategies (N · S · 2^M must stay below 2^28)",
            );
        }
        let ev = &self.evolution;
        check(
            ev.every >= 1 && unit(ev.strategy_mutation) && unit(ev.memory_mutation),
            "evolution",
            "every must be at least 1 and the mutation chances between 0 and 1",
        );
        check(
            self.stop_at <= 1_000_000,
            "stop_at",
            "must be at most 1000000",
        );
        if e.is_empty() {
            Ok(())
        } else {
            Err(e)
        }
    }

    /// Fields that change only on reset and differ in `next`.
    pub(crate) fn structural_changes(&self, next: &FarolConfig) -> Vec<FieldError> {
        let mut out = Vec::new();
        for (field, same) in [
            ("game", self.game == next.game),
            ("agents", self.agents == next.agents),
            ("strategies", self.strategies == next.strategies),
            ("capacity", self.capacity == next.capacity),
            ("scoring", self.scoring == next.scoring),
            ("shared", self.shared == next.shared),
            ("memory", self.memory == next.memory),
            ("mixed_memory", self.mixed_memory == next.mixed_memory),
            ("bias", self.bias == next.bias),
            ("evolution", self.evolution == next.evolution),
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
            "Game",
            "game",
            "Game",
            &[
                ("el_farol", "El Farol (Arthur)"),
                ("minority", "The minority game (Challet & Zhang)"),
            ],
            Reset,
        ),
        Param::integer("Game", "agents", "Agents (N)", (3, 2001), Reset)
            .with_help("Arthur: 100. Challet & Zhang: an odd number, 101 or 1001."),
        Param::integer("Game", "strategies", "Strategies each", (1, LIBRARY), Reset)
            .with_help("Arthur's k (6, 12 or 23), or Challet & Zhang's S (2 to 16)."),
        Param::integer("Game", "capacity", "Capacity (L)", (0, 2001), Reset)
            .nullable()
            .with_help(
                "El Farol: L or more is crowded. Minority game: attending wins at L or fewer. Empty: the game's own, 60 % of N or (N − 1)/2 (the plain game).",
            ),
        Param::choice(
            "Game",
            "behavior",
            "Agents decide",
            &[
                ("inductive", "By their best strategy"),
                ("random", "At random (probability L/N; a fair coin in the plain minority game)"),
            ],
            Live,
        )
        .with_help("Challet, Marsili & Ottino: random agents also average L — the question is how far attendance swings."),
        Param::choice(
            "El Farol",
            "scoring",
            "Predictors rated by",
            &[
                ("error", "Accuracy (Arthur)"),
                ("payoff", "The advice they give (Challet, Marsili & Ottino)"),
            ],
            Reset,
        )
        .shown_if("game", "el_farol"),
        Param::number("El Farol", "decay", "Accuracy memory (λ)", (0.0, 1.0, 0.01), Live)
            .shown_if("game", "el_farol")
            .and_shown_if("scoring", "error")
            .with_help("Error ← λ·error + (1 − λ)·|forecast − attendance|. Arthur gives no value."),
        Param::choice(
            "El Farol",
            "at_capacity",
            "Forecast exactly L",
            &[("stay", "Stay home (Arthur)"), ("go", "Go")],
            Live,
        )
        .shown_if("game", "el_farol"),
        Param::bool("El Farol", "shared", "Everyone holds the whole library", Reset)
            .shown_if("game", "el_farol")
            .with_help("Arthur: 'The reader might ponder what would happen if all agents shared the same set of predictors.'"),
        Param::integer("Minority game", "memory", "Memory (M)", (1, MAX_MEMORY), Reset)
            .shown_if("game", "minority"),
        Param::bool("Minority game", "mixed_memory.enabled", "Mixed memories", Reset)
            .shown_if("game", "minority"),
        Param::integer("Minority game", "mixed_memory.min", "Shortest", (1, MAX_MEMORY), Reset)
            .shown_if("game", "minority")
            .and_shown_if("mixed_memory.enabled", "true"),
        Param::integer("Minority game", "mixed_memory.max", "Longest", (1, MAX_MEMORY), Reset)
            .shown_if("game", "minority")
            .and_shown_if("mixed_memory.enabled", "true"),
        Param::choice(
            "Minority game",
            "payoff",
            "Winners get",
            &[("step", "A point"), ("inverse", "N/x − 2 points (x winners)")],
            Live,
        )
        .shown_if("game", "minority"),
        Param::choice(
            "Minority game",
            "rounding",
            "N/x − 2",
            &[
                ("nearest", "Rounded to a whole number (the paper)"),
                ("exact", "Exact"),
            ],
            Live,
        )
        .shown_if("game", "minority")
        .and_shown_if("payoff", "inverse"),
        Param::number("Minority game", "bias", "Bias (ā)", (0.0, 1.0, 0.01), Reset)
            .shown_if("game", "minority")
            .with_help("The chance a strategy says attend (Challet, Marsili & Ottino)."),
        Param::choice(
            "Minority game",
            "information",
            "History",
            &[("true", "The real one"), ("random", "Random each round")],
            Live,
        )
        .shown_if("game", "minority"),
        Param::bool("Evolution", "evolution.enabled", "Replace the worst", Reset)
            .shown_if("game", "minority")
            .with_help("Challet & Zhang: the worst player is replaced by a copy of the best, its scores reset."),
        Param::integer("Evolution", "evolution.every", "Every (rounds)", (1, 100_000), Reset)
            .shown_if("game", "minority")
            .and_shown_if("evolution.enabled", "true"),
        Param::number(
            "Evolution",
            "evolution.strategy_mutation",
            "A strategy redrawn",
            (0.0, 1.0, 0.01),
            Reset,
        )
        .shown_if("game", "minority")
        .and_shown_if("evolution.enabled", "true"),
        Param::number(
            "Evolution",
            "evolution.memory_mutation",
            "Memory ± 1",
            (0.0, 1.0, 0.01),
            Reset,
        )
        .shown_if("game", "minority")
        .and_shown_if("evolution.enabled", "true"),
        Param::integer("Stopping", "stop_at", "Stop at round", (0, 1_000_000), Live)
            .with_help("0: never."),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ModelConfig, ModelWorld};

    #[test]
    fn defaults_are_arthur_s_bar() {
        let c = FarolConfig::default();
        assert_eq!(
            (c.game, c.agents, c.capacity(), c.strategies),
            (Game::ElFarol, 100, 60, 12)
        );
        assert_eq!(
            (c.scoring, c.at_capacity),
            (Scoring::Error, AtCapacity::Stay)
        );
        assert!(c.validate().is_ok());
    }

    #[test]
    fn validation_names_fields() {
        let bad = FarolConfig {
            agents: 2,
            strategies: 49,
            capacity: Some(3),
            decay: 1.5,
            bias: -0.1,
            memory: 0,
            mixed_memory: MixedMemory {
                enabled: true,
                min: 5,
                max: 3,
            },
            evolution: Evolution {
                every: 0,
                ..Evolution::default()
            },
            stop_at: 2_000_000,
            ..FarolConfig::default()
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
                "agents",
                "strategies",
                "capacity",
                "decay",
                "bias",
                "memory",
                "mixed_memory",
                "evolution",
                "stop_at"
            ]
        );
    }

    #[test]
    fn the_minority_game_keeps_its_tables_within_budget() {
        let mg = |agents, strategies, memory| FarolConfig {
            game: Game::Minority,
            agents,
            strategies,
            memory,
            capacity: None,
            ..FarolConfig::default()
        };
        assert!(mg(1001, 5, 12).validate().is_ok());
        assert!(mg(1001, 5, 12).plain_minority());
        assert_eq!(mg(2001, 16, 3).memory_ceiling(), 13);
        assert_eq!(mg(2001, 16, 16).validate().unwrap_err()[0].field, "memory");
        assert_eq!(mg(101, 2, 3).memory_ceiling(), 16);
        let el = FarolConfig {
            strategies: 17,
            ..FarolConfig::default()
        };
        assert!(
            el.validate().is_ok(),
            "El Farol allows up to the library's 48"
        );
        assert_eq!(
            FarolConfig {
                game: Game::Minority,
                strategies: 17,
                capacity: Some(49),
                ..el
            }
            .validate()
            .unwrap_err()[0]
                .field,
            "strategies"
        );
    }

    #[test]
    fn a_blank_capacity_is_the_game_s_own() {
        let mg = |agents| FarolConfig {
            game: Game::Minority,
            agents,
            capacity: None,
            ..FarolConfig::default()
        };
        assert_eq!(mg(1001).capacity(), 500);
        assert!(mg(501).plain_minority(), "editing N keeps the plain game");
        assert_eq!(mg(501).capacity(), 250);
        let bar = |agents| FarolConfig {
            agents,
            capacity: None,
            ..FarolConfig::default()
        };
        assert_eq!(bar(100).capacity(), 60);
        assert_eq!(bar(201).capacity(), 121, "60 % of N, rounded");
        let fixed = FarolConfig {
            capacity: Some(40),
            ..bar(200)
        };
        assert_eq!(fixed.capacity(), 40);
        let json = serde_json::to_value(mg(101)).unwrap();
        assert!(json["capacity"].is_null());
        let back: FarolConfig = serde_json::from_value(json).unwrap();
        assert_eq!(back, mg(101));
    }

    #[test]
    fn scoring_changes_only_on_reset() {
        let next = FarolConfig {
            scoring: Scoring::Payoff,
            ..FarolConfig::default()
        };
        let changes = FarolConfig::default().structural_changes(&next);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].field, "scoring");
    }

    /// Whether the panel shows `p` for `c`: every condition holds.
    fn shown(p: &Param, c: &FarolConfig) -> bool {
        let json = serde_json::to_value(c).unwrap();
        let holds = |cond: &Option<crate::schema::ShowIf>| {
            cond.is_none_or(|cond| {
                let v = cond.path.split('.').fold(&json, |v, key| &v[key]);
                let s = match v {
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                s == cond.equals
            })
        };
        holds(&p.show_if) && holds(&p.also_if)
    }

    #[test]
    fn fields_show_only_under_their_game() {
        let everything_on = |game| FarolConfig {
            game,
            scoring: Scoring::Error,
            payoff: Payoff::Inverse,
            mixed_memory: MixedMemory {
                enabled: true,
                ..MixedMemory::default()
            },
            evolution: Evolution {
                enabled: true,
                ..Evolution::default()
            },
            ..FarolConfig::default()
        };
        let (el, mg) = (everything_on(Game::ElFarol), everything_on(Game::Minority));
        for p in schema() {
            match p.group {
                "El Farol" => assert!(!shown(&p, &mg), "{} under the minority game", p.path),
                "Minority game" | "Evolution" => {
                    assert!(!shown(&p, &el), "{} under El Farol", p.path)
                }
                _ => {}
            }
        }
        let decay = schema().into_iter().find(|p| p.path == "decay").unwrap();
        assert!(shown(&decay, &el));
        let payoff_rated = FarolConfig {
            scoring: Scoring::Payoff,
            ..el
        };
        assert!(!shown(&decay, &payoff_rated));
    }

    #[test]
    fn schema_paths_exist_and_match_what_set_config_allows() {
        // A set capacity, so the check has a number to change.
        let config = ModelConfig::Farol(FarolConfig {
            capacity: Some(60),
            ..FarolConfig::default()
        });
        crate::schema::check_schema(&schema(), &config, || {
            ModelWorld::new(config.clone(), 1).unwrap()
        });
    }
}
