//! Configs and share links saved before N goods still load and run
//! unchanged. The fixtures were written by this file's (since deleted)
//! `write_legacy_fixtures` before any N-goods change; never regenerate them.

use std::collections::BTreeMap;

use sugarscape_core::config::Config;
use sugarscape_core::presets;
use sugarscape_core::world::World;

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

fn base64url_decode(text: &str) -> Vec<u8> {
    let (mut out, mut acc, mut bits) = (Vec::new(), 0u32, 0u32);
    for c in text.bytes() {
        let v = ALPHABET
            .iter()
            .position(|&a| a == c)
            .expect("base64url character") as u32;
        acc = (acc << 6) | v;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
            acc &= (1 << bits) - 1;
        }
    }
    out
}

/// Fingerprint of the legacy share link's world after 200 ticks, recorded
/// before N goods.
const LEGACY_SHARE_FINGERPRINT: u64 = 0xb80e0bc5c62c69c4;

#[test]
fn legacy_preset_configs_load_as_the_presets() {
    let fixtures: BTreeMap<String, serde_json::Value> =
        serde_json::from_str(include_str!("fixtures/legacy-presets.json")).unwrap();
    assert_eq!(
        fixtures.len(),
        23,
        "the presets that existed before N goods"
    );
    // Presets deliberately changed since: the old config still loads, and
    // differs from today's preset only in the named field.
    // ii-6-waves: moved to the book's full 20×20 block (the model survey).
    let changed_placement = ["ii-6-waves"];
    for (id, json) in fixtures {
        let preset = presets::by_id(&id).unwrap_or_else(|| panic!("preset {id} disappeared"));
        let mut loaded =
            Config::from_json(&json.to_string()).unwrap_or_else(|e| panic!("{id}: {e:?}"));
        if changed_placement.contains(&id.as_str()) {
            assert_ne!(loaded.placement, preset.config.placement, "{id}");
            loaded.placement = preset.config.placement.clone();
        }
        assert_eq!(loaded, preset.config, "{id}");
    }
}

#[test]
fn legacy_share_link_reproduces_its_run() {
    let wire: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/legacy-share.json")).unwrap();
    assert_eq!(wire["v"], 1);
    let config = Config::from_json(&wire["c"].to_string()).unwrap();
    let seed = wire["s"].as_u64().unwrap();
    let caps: Vec<f64> = base64url_decode(wire["l"].as_str().unwrap())
        .into_iter()
        .map(f64::from)
        .collect();
    assert_eq!(caps.len(), 2500);
    let mut w = World::with_capacities(config, seed, Some(&caps)).unwrap();
    w.run(200);
    assert_eq!(w.fingerprint(), LEGACY_SHARE_FINGERPRINT);
}
