//! Algorithmic Collusion against the authors' own code: under their readings
//! (random ties, their RAN2 seeded −session, their cap), a session reaches
//! the same strategies in the same period as their Fortran does
//! (tests/fixtures/calvano-sessions.json, from their replication package).

use sugarscape_core::collusion::{as_coded, CollusionConfig, CollusionWorld};

#[derive(serde::Deserialize)]
struct Fixture {
    sessions: Vec<Session>,
}

#[derive(serde::Deserialize)]
struct Session {
    session: u64,
    periods: u64,
    cycle: usize,
    profits: [f64; 2],
    strategies: Vec<u8>,
}

fn fixture() -> Fixture {
    serde_json::from_str(include_str!("fixtures/calvano-sessions.json")).unwrap()
}

fn check(s: &Session) {
    let mut c = CollusionConfig::default();
    as_coded(&mut c);
    let mut w = CollusionWorld::new(c, s.session).unwrap();
    while !w.is_finished() {
        w.run(1_000_000);
    }
    let o = w.outcome().unwrap();
    assert!(o.converged, "session {}", s.session);
    assert_eq!(o.periods, s.periods, "session {}: periods", s.session);
    let ours: Vec<u8> = (0..225)
        .flat_map(|st| [o.strategies[0][st], o.strategies[1][st]])
        .collect();
    assert_eq!(ours, s.strategies, "session {}: strategies", s.session);
    assert_eq!(o.cycle.len(), s.cycle, "session {}: cycle", s.session);
    for i in 0..2 {
        assert!(
            (o.cycle.profits[i] - s.profits[i]).abs() < 1e-12,
            "session {}: firm {i}'s profit {} against {}",
            s.session,
            o.cycle.profits[i],
            s.profits[i]
        );
    }
}

#[test]
fn the_first_sessions_are_the_authors_period_for_period() {
    for s in fixture().sessions.iter().take(3) {
        check(s);
    }
}

/// All 100 (about a minute in release): `cargo test -p sugarscape-core
/// --release --test collusion -- --ignored`.
#[test]
#[ignore]
fn every_fixture_session_is_the_authors_period_for_period() {
    for s in &fixture().sessions {
        check(s);
    }
}
