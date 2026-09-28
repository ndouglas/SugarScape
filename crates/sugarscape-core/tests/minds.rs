//! Minds 1's reduction: the utility mind with rule M's single consideration
//! (travel 0, crowding 0, idle stay) is rule M. Every golden Sugarscape
//! preset without combat gives the same fingerprint under either rule.

use sugarscape_core::config::{Config, DecisionRule, Idle};
use sugarscape_core::presets;
use sugarscape_core::world::World;

fn fingerprint(config: Config) -> u64 {
    let mut w = World::new(config, 1).unwrap();
    w.run(200);
    w.fingerprint()
}

#[test]
fn the_utility_mind_with_rule_ms_consideration_is_rule_m() {
    let mut checked = 0;
    for p in presets::all() {
        // Only a book-rule preset's own fingerprint is the reduction's
        // target; a preset already tuned under the utility mind (its own
        // travel, crowding or wander) is not rule M and is skipped here.
        if p.config.combat.enabled || p.config.decision.rule != DecisionRule::Book {
            continue;
        }
        let mut c = p.config.clone();
        c.decision.rule = DecisionRule::Utility;
        c.decision.travel = 0.0;
        c.decision.crowding = 0.0;
        c.decision.idle = Idle::Stay;
        assert_eq!(fingerprint(c), fingerprint(p.config.clone()), "{}", p.id);
        checked += 1;
    }
    assert!(checked >= 20, "checked {checked} presets");
}

#[test]
fn walking_at_vision_one_is_jumping() {
    let mut checked = 0;
    for p in presets::all() {
        if p.config.combat.enabled {
            continue;
        }
        let mut jump = p.config.clone();
        jump.vision = sugarscape_core::config::URange::new(1, 1);
        // Minds 2's walk-* presets already start in walk mode, so both sides
        // must set `mode` explicitly to compare a real jump against a real
        // walk (fixed here; it previously compared walk against walk for
        // those presets).
        jump.movement.mode = sugarscape_core::config::MoveMode::Jump;
        // Minds 3's mem-* presets have span > 0, which requires walking;
        // forcing `mode: jump` above would make the config invalid. Memory
        // isn't the subject of this reduction, so turn it off on both sides.
        jump.memory.span = 0;
        let mut walk = jump.clone();
        walk.movement.mode = sugarscape_core::config::MoveMode::Walk;
        assert_eq!(fingerprint(walk), fingerprint(jump), "{}", p.id);
        checked += 1;
    }
    assert!(checked >= 20, "checked {checked} presets");
}

#[test]
fn memory_and_truffles_off_explicitly_is_every_preset() {
    // Minds 3's reduction: a preset's JSON with `memory.span` 0 and
    // `truffles.share` 0 written in explicitly loads (through serde and
    // validation) to a world whose 200-tick fingerprint is the preset's own,
    // which `tests/golden.rs` pins. Memory off draws nothing.
    let mut checked = 0;
    for p in presets::all() {
        // A Minds 3 preset turns memory or truffles on: it's not the book.
        if p.config.memory.span > 0 || p.config.truffles.share > 0.0 {
            continue;
        }
        let mut v = serde_json::to_value(&p.config).unwrap();
        v["memory"]["span"] = serde_json::json!(0);
        v["truffles"]["share"] = serde_json::json!(0.0);
        let c = Config::from_json(&v.to_string()).unwrap_or_else(|e| panic!("{}: {e:?}", p.id));
        assert_eq!((c.memory.span, c.truffles.share), (0, 0.0), "{}", p.id);
        assert_eq!(fingerprint(c), fingerprint(p.config.clone()), "{}", p.id);
        checked += 1;
    }
    assert!(checked >= 20, "checked {checked} presets");
}
