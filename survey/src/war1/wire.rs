//! Bounded, strict W1 input without altering the ordinary Config decoder.
use serde::{
    de::{self, MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer,
};
use std::{collections::BTreeSet, fmt};
use sugarscape_core::{
    presets,
    war::{config::EngagementConfig, Capture, StudyInput},
};

#[derive(Debug, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub enum Request {
    BookC {
        preset: String,
        seed: u64,
        max_steps: u64,
        capture_steps: Vec<u64>,
    },
    ReciprocalGraph {
        config: EngagementConfig,
        seed: u64,
        capture_steps: Vec<u64>,
    },
}

// Validate the original tokens, not rounded values. Integers are later decoded directly as u64.
pub fn reject_unrepresentable_numbers(bytes: &[u8]) -> Result<(), String> {
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'"' {
            index += 1;
            while index < bytes.len() {
                if bytes[index] == b'\\' {
                    index += 2;
                } else if bytes[index] == b'"' {
                    index += 1;
                    break;
                } else {
                    index += 1;
                }
            }
        } else if bytes[index] == b'-' || bytes[index].is_ascii_digit() {
            let start = index;
            index += 1;
            while index < bytes.len()
                && matches!(bytes[index], b'0'..=b'9' | b'.' | b'e' | b'E' | b'+' | b'-')
            {
                index += 1;
            }
            let token = std::str::from_utf8(&bytes[start..index]).map_err(|e| e.to_string())?;
            let value: f64 = token
                .parse()
                .map_err(|_| format!("invalid number literal {token}"))?;
            let nonzero = token
                .split(['e', 'E'])
                .next()
                .unwrap()
                .bytes()
                .any(|b| matches!(b, b'1'..=b'9'));
            if !value.is_finite() || (value == 0.0 && nonzero) {
                return Err(format!(
                    "number literal {token} is not representable as finite nonzero binary64"
                ));
            }
        } else {
            index += 1;
        }
    }
    Ok(())
}

struct UniqueJson;
impl<'de> Deserialize<'de> for UniqueJson {
    fn deserialize<D: Deserializer<'de>>(decoder: D) -> Result<Self, D::Error> {
        struct UniqueVisitor;
        impl<'de> Visitor<'de> for UniqueVisitor {
            type Value = UniqueJson;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("JSON with unique fields")
            }
            fn visit_bool<E: de::Error>(self, _: bool) -> Result<Self::Value, E> {
                Ok(UniqueJson)
            }
            fn visit_i64<E: de::Error>(self, _: i64) -> Result<Self::Value, E> {
                Ok(UniqueJson)
            }
            fn visit_u64<E: de::Error>(self, _: u64) -> Result<Self::Value, E> {
                Ok(UniqueJson)
            }
            fn visit_f64<E: de::Error>(self, _: f64) -> Result<Self::Value, E> {
                Ok(UniqueJson)
            }
            fn visit_str<E: de::Error>(self, _: &str) -> Result<Self::Value, E> {
                Ok(UniqueJson)
            }
            fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
                Ok(UniqueJson)
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                while seq.next_element::<UniqueJson>()?.is_some() {}
                Ok(UniqueJson)
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut seen = BTreeSet::new();
                while let Some(key) = map.next_key::<String>()? {
                    if !seen.insert(key.clone()) {
                        return Err(de::Error::custom(format!("duplicate JSON field: {key}")));
                    }
                    map.next_value::<UniqueJson>()?;
                }
                Ok(UniqueJson)
            }
        }
        decoder.deserialize_any(UniqueVisitor)
    }
}
pub fn decode(bytes: &[u8]) -> Result<Request, String> {
    if bytes.len() > super::io::INPUT_LIMIT {
        return Err("input exceeds 1 MiB".into());
    }
    reject_unrepresentable_numbers(bytes)?;
    let _: UniqueJson = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    serde_json::from_slice(bytes).map_err(|e| e.to_string())
}
pub fn resolve(request: Request) -> Result<(StudyInput, Capture), String> {
    let (input, capture_steps, horizon) = match request {
        Request::BookC {
            preset,
            seed,
            max_steps,
            capture_steps,
        } => {
            let config = presets::by_id(&preset)
                .ok_or_else(|| format!("unknown preset {preset}"))?
                .config;
            config
                .validate()
                .map_err(|e| format!("invalid preset: {e:?}"))?;
            if !(1..=1_000_000).contains(&max_steps) {
                return Err("max_steps must be between 1 and 1000000".into());
            }
            let sites = u64::from(config.width) * u64::from(config.height);
            if sites > 4096
                || !matches!(sites.checked_mul(max_steps), Some(work) if work <= 64_000_000)
            {
                return Err(
                    "Book controls require at most 4096 sites and 64000000 site-ticks".into(),
                );
            }
            (
                StudyInput::BookC {
                    config,
                    seed,
                    max_steps,
                },
                capture_steps,
                max_steps,
            )
        }
        Request::ReciprocalGraph {
            config,
            seed,
            capture_steps,
        } => {
            config
                .validate()
                .map_err(|e| format!("invalid engagement: {e:?}"))?;
            let horizon = config.max_steps;
            (
                StudyInput::ReciprocalGraph { config, seed },
                capture_steps,
                horizon,
            )
        }
    };
    if capture_steps.len() > 1024 {
        return Err("at most 1024 capture requests are permitted".into());
    }
    let mut seen = BTreeSet::new();
    for &step in &capture_steps {
        if step > horizon {
            return Err(format!("capture step {step} exceeds horizon {horizon}"));
        }
        if !seen.insert(step) {
            return Err(format!("duplicate capture step {step}"));
        }
    }
    Ok((
        input,
        Capture {
            retain_steps: capture_steps,
        },
    ))
}
