//! Complete import comparison with the explicitly approved legacy computed-float paths.
use super::{EpisodeRecord, StudyId};
use serde_json::{Number, Value};
const LEGACY_FLOAT_TOLERANCE: f64 = 1e-12;

pub(super) fn records_equivalent(left: &EpisodeRecord, right: &EpisodeRecord) -> bool {
    if left.kind != right.kind
        || left.version != right.version
        || left.study != right.study
        || left.rules_identity != right.rules_identity
        || left.semantics != right.semantics
        || left.checkpoints.len() != right.checkpoints.len()
    {
        return false;
    }
    let same = |mut path: Vec<String>, a: &Value, b: &Value| {
        values_equivalent(left.study, &mut path, a, b)
    };
    if !same(vec!["input".into()], &left.input, &right.input) {
        return false;
    }
    for (a, b) in left.checkpoints.iter().zip(&right.checkpoints) {
        if a.index != b.index || a.kind != b.kind || a.local.len() != b.local.len() {
            return false;
        }
        let prefix = vec!["checkpoints".into(), a.index.to_string()];
        for (key, av, bv) in [
            ("clock", &a.clock, &b.clock),
            ("public", &a.public, &b.public),
        ] {
            let mut path = prefix.clone();
            path.push(key.into());
            if !same(path, av, bv) {
                return false;
            }
        }
        for (agent, local) in &a.local {
            let Some(other) = b.local.get(agent) else {
                return false;
            };
            let mut path = prefix.clone();
            path.extend(["local".into(), agent.clone()]);
            if !same(path, local, other) {
                return false;
            }
        }
        match (&a.researcher, &b.researcher) {
            (None, None) => {}
            (Some(av), Some(bv)) => {
                let mut path = prefix;
                path.push("researcher".into());
                if !same(path, av, bv) {
                    return false;
                }
            }
            _ => return false,
        }
    }
    same(vec!["payload".into()], &left.payload, &right.payload)
}
fn values_equivalent(study: StudyId, path: &mut Vec<String>, left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Number(a), Value::Number(b)) => {
            numbers_equal(a, b)
                || (computed_float_path(study, path)
                    && (a.as_f64().unwrap() - b.as_f64().unwrap()).abs() <= LEGACY_FLOAT_TOLERANCE)
        }
        (Value::Array(a), Value::Array(b)) => {
            if a.len() != b.len() {
                return false;
            }
            for (index, (av, bv)) in a.iter().zip(b).enumerate() {
                path.push(index.to_string());
                let same = values_equivalent(study, path, av, bv);
                path.pop();
                if !same {
                    return false;
                }
            }
            true
        }
        (Value::Object(a), Value::Object(b)) => {
            if a.len() != b.len() {
                return false;
            }
            for (key, av) in a {
                let Some(bv) = b.get(key) else {
                    return false;
                };
                path.push(key.clone());
                let same = values_equivalent(study, path, av, bv);
                path.pop();
                if !same {
                    return false;
                }
            }
            true
        }
        _ => left == right,
    }
}
// Decimal spelling equivalence must not round a large integer into another value.
fn numbers_equal(a: &Number, b: &Number) -> bool {
    if a == b {
        return true;
    }
    if a.is_f64() && b.is_f64() {
        return a.as_f64() == b.as_f64();
    }
    if a.is_f64() {
        float_integer_equal(a.as_f64().unwrap(), b)
    } else if b.is_f64() {
        float_integer_equal(b.as_f64().unwrap(), a)
    } else {
        false
    }
}
fn float_integer_equal(value: f64, integer: &Number) -> bool {
    if value.fract() != 0.0 {
        return false;
    }
    if let Some(unsigned) = integer.as_u64() {
        (0.0..18446744073709551616.0).contains(&value) && value as u64 == unsigned
    } else if let Some(signed) = integer.as_i64() {
        (-9223372036854775808.0..9223372036854775808.0).contains(&value) && value as i64 == signed
    } else {
        false
    }
}
fn index(part: &str) -> bool {
    !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit())
}
fn snapshot_probability(path: &[&str]) -> bool {
    match path {
        ["hypotheses", i, "probability"] | ["propositions", i, "probability_true"] => index(i),
        ["speakers", i, "profiles", j, "probability"] => index(i) && index(j),
        _ => false,
    }
}
fn computed_float_path(study: StudyId, path: &[String]) -> bool {
    let parts: Vec<_> = path.iter().map(String::as_str).collect();
    match (study, parts.as_slice()) {
        (StudyId::Testimony, ["payload", "snapshot", tail @ ..]) => snapshot_probability(tail),
        (StudyId::Testimony, ["checkpoints", i, "local", "listener", "snapshot", tail @ ..])
        | (StudyId::Testimony, ["checkpoints", i, "researcher", "snapshot", tail @ ..])
            if index(i) =>
        {
            snapshot_probability(tail)
        }
        (StudyId::TestimonyGame, ["payload", "decision", "posterior_true"])
        | (StudyId::TestimonyGame, ["payload", "conditional_regret"]) => true,
        (
            StudyId::TestimonyGame,
            ["checkpoints", i, "local", "2", "decision", "posterior_true"],
        ) if index(i) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn numeric_spelling_equivalence_never_rounds_distinct_large_integers() {
        for (integer, float, equal) in [
            (json!(3u64), json!(3.0), true),
            (json!(9007199254740992u64), json!(9007199254740992.0), true),
            (json!(9007199254740993u64), json!(9007199254740992.0), false),
            (json!(u64::MAX), json!(18446744073709551616.0), false),
            (json!(i64::MAX), json!(9223372036854775808.0), false),
            (json!(i64::MIN), json!(-9223372036854775808.0), true),
        ] {
            let a = integer.as_number().unwrap();
            let b = float.as_number().unwrap();
            assert_eq!(numbers_equal(a, b), equal, "{integer} vs {float}");
            assert_eq!(numbers_equal(b, a), equal, "{float} vs {integer}");
        }
    }
}
