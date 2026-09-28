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
        let mut walk = jump.clone();
        walk.movement.mode = sugarscape_core::config::MoveMode::Walk;
        assert_eq!(fingerprint(walk), fingerprint(jump), "{}", p.id);
        checked += 1;
    }
    assert!(checked >= 20, "checked {checked} presets");
}

#[test]
fn memory_and_truffles_off_explicitly_is_every_preset() {
    // Minds 3's reduction: with `memory.span` 0 and `truffles.share` 0 set
    // explicitly, memory is off and draws nothing, so every walking preset,
    // and every other non-combat preset, keeps its own golden fingerprint.
    let mut checked = 0;
    for p in presets::all() {
        if p.config.combat.enabled {
            continue;
        }
        // A Minds 3 preset turns memory or truffles on: it's not the book.
        if p.config.memory.span > 0 || p.config.truffles.share > 0.0 {
            continue;
        }
        let mut c = p.config.clone();
        c.memory.span = 0;
        c.truffles.share = 0.0;
        // `tests/golden.rs` pins each preset's own fingerprint.
        assert_eq!(fingerprint(c), fingerprint(p.config.clone()), "{}", p.id);
        checked += 1;
    }
    assert!(checked >= 20, "checked {checked} presets");
}
