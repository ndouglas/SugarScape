//! Build-time source binding for the F5 collector; no Git or simulation is needed to build.
#[path = "build_support/f5_source_identity.rs"]
mod source_identity;
use std::{fs, path::Path};

fn read_inputs(
    root: &Path,
    relative: &Path,
    entries: &mut Vec<(String, Vec<u8>)>,
) -> Result<(), String> {
    let path = root.join(relative);
    let metadata = match fs::symlink_metadata(&path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(format!("{}: {error}", path.display())),
    };
    if metadata.is_dir() {
        for entry in fs::read_dir(&path).map_err(|e| format!("{}: {e}", path.display()))? {
            let entry = entry.map_err(|e| format!("{}: {e}", path.display()))?;
            read_inputs(root, &relative.join(entry.file_name()), entries)?;
        }
    } else if metadata.is_file() {
        let name = relative
            .to_str()
            .ok_or("source path must be UTF-8")?
            .replace('\\', "/");
        if source_identity::selected(&name) {
            entries.push((
                name,
                fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?,
            ));
        }
    } else {
        return Err(format!(
            "source input must be a regular file or directory: {}",
            path.display()
        ));
    }
    Ok(())
}

fn main() {
    let manifest = std::env::var_os("CARGO_MANIFEST_DIR").expect("Cargo manifest directory");
    let root = Path::new(&manifest).parent().expect("repository parent");
    let mut entries = Vec::new();
    for relative in source_identity::DIRECTORIES
        .iter()
        .chain(source_identity::FILES)
    {
        println!("cargo:rerun-if-changed={}", root.join(relative).display());
        read_inputs(root, Path::new(relative), &mut entries).expect("F5 build source identity");
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    entries.dedup_by(|a, b| a.0 == b.0);
    let identity =
        source_identity::fingerprint(entries.into_iter().map(Ok)).expect("F5 source hash");
    let out = std::env::var_os("OUT_DIR").expect("Cargo output directory");
    fs::write(
        Path::new(&out).join("f5_source_identity.rs"),
        format!("const COMPILED_SOURCE_IDENTITY: &str = {identity:?};\n"),
    )
    .expect("write F5 compiled source identity");
    let entries = bt_inputs::read(root).expect("BT compiled inputs");
    for relative in bt_inputs::DIRECTORIES.iter().chain(bt_inputs::FILES) {
        println!("cargo:rerun-if-changed={}", root.join(relative).display());
    }
    let present = entries.iter().any(|e| e.0 == bt_inputs::PROTOCOL);
    let paths: Vec<_> = entries.iter().map(|e| e.0.as_str()).collect();
    let identity =
        bt_inputs::fingerprint(entries.clone().into_iter().map(Ok)).expect("BT source hash");
    let compiler = std::process::Command::new(std::env::var_os("RUSTC").expect("RUSTC"))
        .arg("-vV")
        .output()
        .expect("BT compiler identity");
    assert!(compiler.status.success(), "BT rustc -vV failed");
    let compiler = String::from_utf8(compiler.stdout).expect("UTF-8 compiler identity");
    let flags: Vec<_> = [
        "RUSTFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
        "TARGET",
        "HOST",
        "PROFILE",
        "OPT_LEVEL",
        "DEBUG",
    ]
    .iter()
    .map(|name| ((*name).to_owned(), std::env::var(name).unwrap_or_default()))
    .collect();
    fs::write(Path::new(&out).join("bt_compiled_inputs.rs"), format!(
        "const COMPILED_BT_INPUTS_SHA256: &str = {identity:?};\nconst COMPILED_BT_INPUT_PATHS: &[&str] = &{paths:?};\nconst COMPILED_BT_PROTOCOL_PRESENT: bool = {present};\nconst COMPILED_BT_TOOLCHAIN: &str = {compiler:?};\nconst COMPILED_BT_FLAGS: &[(&str,&str)] = &{flags:?};\n"
    )).expect("write BT compiled inputs");
}

#[path = "build_support/bt_source_identity.rs"]
mod bt_inputs;
