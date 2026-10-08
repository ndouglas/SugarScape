//! Lossless study payloads; envelope counts and normalized input are separate.
use super::FieldError;
use serde::Serialize;
use serde_json::Value;

/// Preserve signed/unsigned integer atoms as decimal strings and floats as floats.
pub fn lossless_value<T: Serialize>(value: &T) -> Result<Value, Vec<FieldError>> {
    let mut value =
        serde_json::to_value(value).map_err(|e| super::error("payload", e.to_string()))?;
    stringify_integers(&mut value);
    Ok(value)
}
fn stringify_integers(value: &mut Value) {
    match value {
        Value::Number(number) if number.is_i64() || number.is_u64() => {
            *value = Value::String(number.to_string());
        }
        Value::Array(values) => values.iter_mut().for_each(stringify_integers),
        Value::Object(values) => values.values_mut().for_each(stringify_integers),
        _ => {}
    }
}
