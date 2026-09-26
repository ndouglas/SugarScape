//! Parameter schemas (milestone 9): each model other than the sugarscape
//! describes its config's fields, and the page builds its Rules panel from
//! the description (the sugarscape keeps its bespoke panel).

use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ParamKind {
    /// A whole number.
    Integer,
    /// Any number in `min..=max`, in steps of `step`.
    Number,
    /// A `{ min, max }` pair: the panel sets `<path>.min` and `<path>.max`.
    Range,
    Bool,
    /// One of `choices` (a string).
    Choice,
}

/// Whether a change applies to the running world or rebuilds it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Apply {
    Live,
    Reset,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Choice {
    pub value: &'static str,
    pub label: &'static str,
}

/// One field of a model's config, as the page shows it.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Param {
    pub path: &'static str,
    pub label: &'static str,
    pub kind: ParamKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<f64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub choices: Vec<Choice>,
    pub apply: Apply,
    /// The panel section it is shown in.
    pub group: &'static str,
    /// A one-line explanation shown under the control (milestone 10).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<&'static str>,
    /// Shown only while another field has a value (milestone 11: Model II's
    /// population fields).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_if: Option<ShowIf>,
    /// A number field that may be empty (JSON null): the panel shows null
    /// as an empty box and sends null for one (milestone 14: the
    /// ethnocentrism model's tag mutation).
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub nullable: bool,
}

/// A condition on the config: the field at `path` (a string, or a bool
/// compared as `"true"`/`"false"`) equals `equals`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ShowIf {
    pub path: &'static str,
    pub equals: &'static str,
}

impl Param {
    fn new(
        group: &'static str,
        path: &'static str,
        label: &'static str,
        kind: ParamKind,
        apply: Apply,
    ) -> Self {
        Param {
            path,
            label,
            kind,
            min: None,
            max: None,
            step: None,
            choices: Vec::new(),
            apply,
            group,
            help: None,
            show_if: None,
            nullable: false,
        }
    }

    /// The same field with a one-line explanation.
    pub fn with_help(mut self, help: &'static str) -> Self {
        self.help = Some(help);
        self
    }

    /// The same field, shown only while the field `path` is `equals` (a
    /// bool as `"true"` or `"false"`).
    pub fn shown_if(mut self, path: &'static str, equals: &'static str) -> Self {
        self.show_if = Some(ShowIf { path, equals });
        self
    }

    /// The same field, which may be empty (null).
    pub fn nullable(mut self) -> Self {
        self.nullable = true;
        self
    }

    fn bounded(mut self, min: f64, max: f64, step: f64) -> Self {
        self.min = Some(min);
        self.max = Some(max);
        self.step = Some(step);
        self
    }

    pub fn integer(
        group: &'static str,
        path: &'static str,
        label: &'static str,
        (min, max): (u32, u32),
        apply: Apply,
    ) -> Self {
        Self::new(group, path, label, ParamKind::Integer, apply).bounded(
            f64::from(min),
            f64::from(max),
            1.0,
        )
    }

    pub fn number(
        group: &'static str,
        path: &'static str,
        label: &'static str,
        (min, max, step): (f64, f64, f64),
        apply: Apply,
    ) -> Self {
        Self::new(group, path, label, ParamKind::Number, apply).bounded(min, max, step)
    }

    /// A `{ min, max }` pair whose ends lie in `min..=max`, in steps of `step`.
    pub fn range(
        group: &'static str,
        path: &'static str,
        label: &'static str,
        (min, max, step): (f64, f64, f64),
        apply: Apply,
    ) -> Self {
        Self::new(group, path, label, ParamKind::Range, apply).bounded(min, max, step)
    }

    pub fn bool(
        group: &'static str,
        path: &'static str,
        label: &'static str,
        apply: Apply,
    ) -> Self {
        Self::new(group, path, label, ParamKind::Bool, apply)
    }

    pub fn choice(
        group: &'static str,
        path: &'static str,
        label: &'static str,
        choices: &[(&'static str, &'static str)],
        apply: Apply,
    ) -> Self {
        let mut p = Self::new(group, path, label, ParamKind::Choice, apply);
        p.choices = choices
            .iter()
            .map(|&(value, label)| Choice { value, label })
            .collect();
        p
    }
}

/// Checks a model's schema against its config type (used by each model's
/// tests): every path exists in `config`'s JSON with the kind's shape, and
/// changing it on a running world is accepted exactly when it is `Live`.
#[cfg(test)]
pub(crate) fn check_schema(
    schema: &[Param],
    config: &crate::model::ModelConfig,
    world: impl Fn() -> crate::model::ModelWorld,
) {
    use serde_json::{json, Value};
    let value = serde_json::to_value(config).unwrap();
    for p in schema {
        let at = p
            .path
            .split('.')
            .try_fold(&value, |v, k| v.get(k))
            .unwrap_or_else(|| panic!("{} is not in the config", p.path));
        let changed: Value = match p.kind {
            ParamKind::Integer => json!(at.as_u64().unwrap() + 1),
            // Up a step, or down one from the top of the range.
            ParamKind::Number => {
                let (v, step) = (at.as_f64().unwrap(), p.step.unwrap());
                json!(if v + step <= p.max.unwrap() {
                    v + step
                } else {
                    v - step
                })
            }
            ParamKind::Range => {
                let (lo, hi) = (at["min"].as_f64().unwrap(), at["max"].as_f64().unwrap());
                assert!(lo <= hi, "{}", p.path);
                if at["min"].is_u64() {
                    json!({ "min": lo as u64, "max": hi as u64 + 1 })
                } else {
                    json!({ "min": lo, "max": (hi + p.step.unwrap()).min(p.max.unwrap()) })
                }
            }
            ParamKind::Bool => json!(!at.as_bool().unwrap()),
            ParamKind::Choice => {
                let now = at.as_str().unwrap();
                json!(p.choices.iter().find(|c| c.value != now).unwrap().value)
            }
        };
        let mut next = config.clone();
        for (k, v) in match p.kind {
            ParamKind::Range => vec![
                (format!("{}.min", p.path), changed["min"].clone()),
                (format!("{}.max", p.path), changed["max"].clone()),
            ],
            _ => vec![(p.path.to_string(), changed)],
        } {
            next = next
                .with_path(&k, &v)
                .unwrap_or_else(|e| panic!("{k}: {e:?}"));
        }
        assert_ne!(&next, config, "{} did not change", p.path);
        let mut w = world();
        let result = w.model_mut().set_config(next);
        assert_eq!(
            result.is_ok(),
            p.apply == Apply::Live,
            "{}: {result:?}",
            p.path
        );
    }
}
