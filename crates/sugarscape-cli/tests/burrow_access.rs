//! Checked resource-access CLI boundary.
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use sugarscape_core::burrow::{run_access_episode, AccessConfig, AccessEpisode, RunOptions};

struct Scratch(PathBuf);
impl Scratch {
    fn new(name: &str) -> Self {
        let dir =
            std::env::temp_dir().join(format!("sugarscape-access-{}-{name}", std::process::id()));
        if dir.exists() {
            std::fs::remove_dir_all(&dir).unwrap();
        }
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("input.json"),
            include_str!("../../../docs/examples/burrow/access-known-goal.json"),
        )
        .unwrap();
        Self(dir)
    }
    fn config(&self) -> PathBuf {
        self.0.join("input.json")
    }
    fn out(&self) -> PathBuf {
        self.0.join("out")
    }
    fn run(&self, extra: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_sugarscape"))
            .args([
                "burrow-access",
                "--config",
                path(&self.config()),
                "--out",
                path(&self.out()),
            ])
            .args(extra)
            .output()
            .unwrap()
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}
fn path(p: &Path) -> &str {
    p.to_str().unwrap()
}
fn read(p: &Path) -> String {
    std::fs::read_to_string(p).unwrap()
}
fn success(out: &Output) {
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}
fn error(out: &Output, code: i32, field: &str) {
    assert_eq!(
        out.status.code(),
        Some(code),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        String::from_utf8_lossy(&out.stderr).contains(field),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn both_policies_export_full_width_seed_normalized_outer_records() {
    for policy in ["explore", "known_goal"] {
        let s = Scratch::new(policy);
        let mut config: AccessConfig = serde_json::from_str(&read(&s.config())).unwrap();
        config.task.objective = serde_json::from_str(&format!("\"{policy}\"")).unwrap();
        std::fs::write(s.config(), serde_json::to_string(&config).unwrap()).unwrap();
        let out = s.run(&[
            "--seed",
            "18446744073709551615",
            "--ticks",
            "16",
            "--sample-every",
            "4",
        ]);
        success(&out);
        let actual: AccessEpisode =
            serde_json::from_str(&read(&s.out().join("episode.json"))).unwrap();
        let expected = run_access_episode(
            config.clone(),
            u64::MAX,
            RunOptions {
                ticks: 16,
                sample_every: 4,
            },
        )
        .unwrap();
        assert_eq!(actual, expected);
        assert_eq!(
            serde_json::from_str::<AccessConfig>(&read(&s.out().join("config.json"))).unwrap(),
            config
        );
        let summary: serde_json::Value =
            serde_json::from_str(&read(&s.out().join("summary.json"))).unwrap();
        assert_eq!(
            summary,
            serde_json::json!({ "base": actual.episode.final_summary, "access": actual.access })
        );
        let maps = read(&s.out().join("maps.txt"));
        assert!(maps.starts_with(&format!(
            "task goal=(7,12) objective={policy} goal_weight=3 structural_access="
        )));
        for frame in &actual.episode.frames {
            assert!(maps.contains(&format!(
                "tick {} fingerprint {}\n{}\n",
                frame.tick, frame.fingerprint, frame.ascii
            )));
        }
        assert!(String::from_utf8(out.stdout)
            .unwrap()
            .starts_with(&actual.episode.frames.last().unwrap().ascii));
        assert_eq!(std::fs::read_dir(s.out()).unwrap().count(), 4);
    }
}

#[test]
fn zero_ticks_and_existing_empty_directory_are_accepted() {
    let s = Scratch::new("zero");
    std::fs::create_dir(s.out()).unwrap();
    success(&s.run(&["--ticks", "0"]));
    let actual: AccessEpisode = serde_json::from_str(&read(&s.out().join("episode.json"))).unwrap();
    assert_eq!(actual.access.observed_opportunities, 0);
    assert!(actual.access.deadline_censored);
    assert!(actual.access.first_access.is_none());
    assert_eq!(
        actual.episode.frames[0]
            .ascii
            .lines()
            .nth(12)
            .unwrap()
            .chars()
            .nth(7),
        Some('#')
    );
}

#[test]
fn defaults_are_checked_and_retained() {
    let s = Scratch::new("defaults");
    success(&s.run(&[]));
    let actual: AccessEpisode = serde_json::from_str(&read(&s.out().join("episode.json"))).unwrap();
    assert_eq!(
        (
            actual.episode.seed.as_str(),
            actual.episode.requested_ticks,
            actual.episode.completed_ticks
        ),
        ("7", 512, 512)
    );
    assert_eq!(actual.episode.frames[1].tick, 32);
}

#[test]
fn invalid_task_creates_no_outputs() {
    let s = Scratch::new("invalid");
    let mut config: AccessConfig = serde_json::from_str(&read(&s.config())).unwrap();
    config.task.goal.x = 999;
    std::fs::write(s.config(), serde_json::to_string(&config).unwrap()).unwrap();
    error(&s.run(&[]), 2, "task.goal");
    assert!(!s.out().exists());
}

#[test]
fn malformed_config_has_context_and_creates_no_outputs() {
    let s = Scratch::new("malformed");
    std::fs::write(s.config(), "{invalid").unwrap();
    error(&s.run(&[]), 2, "burrow_access_config");
    assert!(!s.out().exists());
}

#[test]
fn zero_sampling_creates_no_outputs() {
    let s = Scratch::new("sampling");
    error(&s.run(&["--sample-every", "0"]), 2, "sample_every");
    assert!(!s.out().exists());
}

#[test]
fn nonempty_output_is_preserved() {
    let s = Scratch::new("nonempty");
    std::fs::create_dir(s.out()).unwrap();
    std::fs::write(s.out().join("keep.txt"), "keep").unwrap();
    error(&s.run(&["--ticks", "0"]), 1, "not empty");
    assert_eq!(read(&s.out().join("keep.txt")), "keep");
    assert_eq!(std::fs::read_dir(s.out()).unwrap().count(), 1);
}

#[test]
fn output_file_is_preserved() {
    let s = Scratch::new("file");
    std::fs::write(s.out(), "keep").unwrap();
    error(&s.run(&["--ticks", "0"]), 1, "not a directory");
    assert_eq!(read(&s.out()), "keep");
}
