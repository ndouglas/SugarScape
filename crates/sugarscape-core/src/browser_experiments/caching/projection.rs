//! Explicit Agent field allowlists. Research DTOs must never be actor-filtered wholesale.
use crate::{
    browser_experiments::{error, FieldError},
    geometry::{Pos, Torus},
};
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub fn site_position(site: u32) -> Result<Pos, Vec<FieldError>> {
    if site >= 81 {
        return Err(error(
            "episode.site",
            "native site is outside fixed 9x9 arena",
        ));
    }
    Ok(Torus::new(9, 9).pos(site as usize))
}
pub fn agent(
    id: u64,
    pos: Pos,
    holdings: f64,
    caches: &BTreeMap<u32, f64>,
) -> Result<Value, Vec<FieldError>> {
    if !(1..=2).contains(&id) || pos.x >= 9 || pos.y >= 9 || caches.len() > 81 {
        return Err(error(
            "episode.roles",
            "native role exceeds fixed caching profile",
        ));
    }
    let caches = caches
        .iter()
        .map(|(&site, &amount)| Ok(json!({"site":site,"pos":site_position(site)?,"amount":amount})))
        .collect::<Result<Vec<_>, Vec<FieldError>>>()?;
    Ok(json!({"id":id.to_string(),"pos":pos,"holdings":holdings,"caches":caches}))
}
pub fn seen(
    site: u32,
    owner: u64,
    amount: f64,
    tick: u64,
    boundary: u64,
) -> Result<Value, Vec<FieldError>> {
    if !(1..=2).contains(&owner) || tick > boundary {
        return Err(error(
            "episode.seen",
            "native memory owner or evidence clock exceeds fixed profile",
        ));
    }
    Ok(
        json!({"site":site,"pos":site_position(site)?,"owner":owner.to_string(),"amount":amount,"tick":tick.to_string(),"age":(boundary-tick).to_string(),"kind":"remembered_cache"}),
    )
}
pub fn public(lab_kind: &str, terminal: bool) -> Value {
    // Both native rig_config constructors use exactly these opaque perimeter cells.
    // No supplied resource, prepared source, or display-marker locations are public.
    let walls = (0..81)
        .filter(|site| {
            let x = site % 9;
            let y = site / 9;
            x == 0 || x == 8 || y == 0 || y == 8
        })
        .map(|site| json!({"x":site%9,"y":site/9}))
        .collect::<Vec<_>>();
    json!({"arena":{"width":9,"height":9},"layout_kind":"reference_layout","lab_kind":lab_kind,"walls":walls,"terminal":terminal})
}
pub fn clock(tick: u64) -> Value {
    json!({"native_tick":tick.to_string(),"action_interval":if tick==0 {Value::Null} else {json!({"start_tick":(tick-1).to_string(),"end_tick":tick.to_string()})}})
}
