use super::source_identity::*;
fn entries() -> Vec<(String, Vec<u8>)> {
    FILES
        .iter()
        .map(|p| ((*p).into(), vec![1]))
        .chain(REQUIRED_SOURCES.iter().map(|p| ((*p).into(), vec![2])))
        .collect()
}
#[test]
fn stamp_is_order_independent_and_sensitive_to_bytes() {
    let mut files = entries();
    let stamp = fingerprint(files.clone().into_iter().map(Ok)).unwrap();
    files.reverse();
    assert_eq!(
        stamp,
        fingerprint(files.clone().into_iter().map(Ok)).unwrap()
    );
    files[0].1.push(4);
    assert_ne!(stamp, fingerprint(files.into_iter().map(Ok)).unwrap());
}
#[test]
fn stamp_rejects_duplicates_missing_and_unselected_inputs() {
    let mut files = entries();
    files.push(files[0].clone());
    assert!(fingerprint(files.into_iter().map(Ok)).is_err());
    let mut files = entries();
    files.remove(0);
    assert!(fingerprint(files.into_iter().map(Ok)).is_err());
    let mut files = entries();
    files.push(("../outside".into(), vec![]));
    assert!(fingerprint(files.into_iter().map(Ok)).is_err());
}

#[test]
fn stamp_requires_the_local_runner_source_files() {
    for source in [
        "survey/src/war1/mod.rs",
        "survey/src/war1/cli.rs",
        "survey/src/war1/io.rs",
        "survey/src/war1/wire.rs",
    ] {
        let entries = entries()
            .into_iter()
            .filter(|(path, _)| path != source)
            .map(Ok);
        assert!(fingerprint(entries).is_err(), "missing {source}");
    }
}
#[cfg(unix)]
#[test]
fn source_reader_rejects_symlinks() {
    let parent = std::env::temp_dir().canonicalize().unwrap();
    let root = parent.join(format!("war1-source-{}", std::process::id()));
    std::fs::create_dir(&root).unwrap();
    std::fs::create_dir_all(root.join("crates/sugarscape-core")).unwrap();
    std::os::unix::fs::symlink(&parent, root.join("crates/sugarscape-core/src")).unwrap();
    assert!(read(&root).unwrap_err().contains("symlink"));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn runtime_binding_refuses_changed_source_bytes() {
    let root = std::env::temp_dir()
        .canonicalize()
        .unwrap()
        .join(format!("war1-binding-{}", std::process::id()));
    std::fs::create_dir(&root).unwrap();
    let inputs = entries();
    for (path, bytes) in &inputs {
        let file = root.join(path);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, bytes).unwrap();
    }
    let compiled = fingerprint(inputs.into_iter().map(Ok)).unwrap();
    verify(&root, &compiled).unwrap();
    std::fs::write(root.join("survey/src/war1/cli.rs"), b"changed after build").unwrap();
    let result = verify(&root, &compiled);
    std::fs::remove_dir_all(root).unwrap();
    assert!(result.unwrap_err().contains("changed since compilation"));
}
