//! Validate coordinates before permissive core Pos decoding can drop extra keys.
use serde::{de::DeserializeOwned, Deserialize};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StrictPos {
    x: u32,
    y: u32,
}
pub(super) fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, String> {
    fn walk(raw: &serde_json::value::RawValue) -> Result<(), String> {
        let text = raw.get().trim_start();
        if text.starts_with('{') {
            let entries: std::collections::BTreeMap<String, Box<serde_json::value::RawValue>> =
                serde_json::from_str(text).map_err(|e| e.to_string())?;
            if entries.contains_key("x") || entries.contains_key("y") {
                let pos: StrictPos =
                    serde_json::from_str(text).map_err(|e| format!("strict position: {e}"))?;
                if pos.x >= 11 || pos.y >= 11 {
                    return Err("position outside 11-grid".into());
                }
            }
            for v in entries.values() {
                walk(v)?
            }
        } else if text.starts_with('[') {
            let entries: Vec<Box<serde_json::value::RawValue>> =
                serde_json::from_str(text).map_err(|e| e.to_string())?;
            for v in entries {
                walk(&v)?
            }
        }
        Ok(())
    }
    let raw: Box<serde_json::value::RawValue> =
        serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    walk(&raw)?;
    serde_json::from_slice(bytes).map_err(|e| e.to_string())
}
