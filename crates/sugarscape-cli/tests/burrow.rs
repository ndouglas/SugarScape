//! Checked standalone Burrow CLI boundary and replay exports.
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use sugarscape_core::burrow::{Episode, LabConfig, Snapshot};

struct Scratch(PathBuf);
impl Scratch {
    fn new(name: &str) -> Self {
        let dir =
            std::env::temp_dir().join(format!("sugarscape-burrow-{}-{name}", std::process::id()));
        if dir.exists() {
            std::fs::remove_dir_all(&dir).unwrap();
        }
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("input.json"),
            serde_json::to_string(&LabConfig::default()).unwrap(),
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
                "burrow",
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
fn exports_full_u64_seed_normalized_config_summary_and_ordered_maps() {
    let s = Scratch::new("exports");
    let out = s.run(&[
        "--seed",
        "18446744073709551615",
        "--ticks",
        "8",
        "--sample-every",
        "4",
    ]);
    success(&out);
    let episode: Episode = serde_json::from_str(&read(&s.out().join("episode.json"))).unwrap();
    assert_eq!(episode.seed, "18446744073709551615");
    assert_eq!(
        episode.frames.iter().map(|f| f.tick).collect::<Vec<_>>(),
        [0, 4, 8]
    );
    let config: LabConfig = serde_json::from_str(&read(&s.out().join("config.json"))).unwrap();
    assert_eq!(config, episode.config);
    let summary: Snapshot = serde_json::from_str(&read(&s.out().join("summary.json"))).unwrap();
    assert_eq!(summary, episode.final_summary);
    let maps = episode
        .frames
        .iter()
        .map(|f| {
            format!(
                "tick {} fingerprint {}\n{}\n",
                f.tick, f.fingerprint, f.ascii
            )
        })
        .collect::<String>();
    assert_eq!(read(&s.out().join("maps.txt")), maps);
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.starts_with(&episode.frames.last().unwrap().ascii));
    assert!(stdout.contains("opportunities=64"));
    assert_eq!(std::fs::read_dir(s.out()).unwrap().count(), 4);
}

#[test]
fn defaults_and_empty_output_directory_are_accepted() {
    let s = Scratch::new("defaults");
    std::fs::write(s.config(), "{}").unwrap();
    std::fs::create_dir(s.out()).unwrap();
    success(&s.run(&[]));
    let episode: Episode = serde_json::from_str(&read(&s.out().join("episode.json"))).unwrap();
    assert_eq!(episode.config, LabConfig::default());
    assert_eq!(
        (
            episode.seed.as_str(),
            episode.requested_ticks,
            episode.completed_ticks
        ),
        ("7", 512, 512)
    );
    assert_eq!(episode.frames[1].tick, 32);
}

#[test]
fn invalid_config_exits_two_before_creating_output() {
    let s = Scratch::new("invalid");
    let c = LabConfig {
        freshness_window: 0,
        ..Default::default()
    };
    std::fs::write(s.config(), serde_json::to_string(&c).unwrap()).unwrap();
    error(&s.run(&[]), 2, "freshness_window");
    assert!(!s.out().exists());
}

#[test]
fn malformed_config_exits_two_with_context() {
    let s = Scratch::new("malformed");
    std::fs::write(s.config(), "{invalid").unwrap();
    error(&s.run(&[]), 2, "burrow_config");
    assert!(!s.out().exists());
}

#[test]
fn zero_sampling_exits_two_before_creating_output() {
    let s = Scratch::new("sampling");
    error(&s.run(&["--sample-every", "0"]), 2, "sample_every");
    assert!(!s.out().exists());
}

#[test]
fn output_file_exits_one_and_preserves_contents() {
    let s = Scratch::new("file");
    std::fs::write(s.out(), "keep").unwrap();
    error(&s.run(&["--ticks", "0"]), 1, "output directory");
    assert_eq!(read(&s.out()), "keep");
}

#[test]
fn nonempty_directory_exits_one_and_preserves_contents() {
    let s = Scratch::new("nonempty");
    std::fs::create_dir(s.out()).unwrap();
    std::fs::write(s.out().join("keep.txt"), "keep").unwrap();
    error(&s.run(&["--ticks", "0"]), 1, "not empty");
    assert_eq!(std::fs::read_dir(s.out()).unwrap().count(), 1);
    assert_eq!(read(&s.out().join("keep.txt")), "keep");
}

#[test]
fn missing_config_exits_one_before_creating_output() {
    let s = Scratch::new("missing");
    std::fs::remove_file(s.config()).unwrap();
    error(&s.run(&[]), 1, "cannot read");
    assert!(!s.out().exists());
}

#[test]
fn existing_preset_listing_is_unchanged() {
    let out = Command::new(env!("CARGO_BIN_EXE_sugarscape"))
        .arg("presets")
        .output()
        .unwrap();
    success(&out);
    let expected = sugarscape_core::presets::catalog()
        .iter()
        .map(|p| format!("{}\t{}\t{}\t{}\n", p.id, p.source, p.name, p.title()))
        .collect::<String>();
    assert_eq!(String::from_utf8(out.stdout).unwrap(), expected);
}
