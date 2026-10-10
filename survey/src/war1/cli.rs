//! Exact native engineering commands and independent source/binary binding.
use super::{
    io::{self, Journal, JournalReceipt},
    source_identity, wire,
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};
use sugarscape_core::war::{run_to, RunFailure, RunSummary};

include!(concat!(env!("OUT_DIR"), "/war1_compiled_inputs.rs"));
const HELP: &str = "war1: native engineering cases (unregistered)\n\nwar1 validate --input FILE\nwar1 run --input FILE --out DIRECTORY\nwar1 --help\n\nInput must be a regular file (at most 1 MiB). Output must be a new directory\nwith an existing regular parent; symlinks and '..' path components are refused.\nNo batch, collection, retry, overwrite or resume command is available.\n";
pub struct CliFailure {
    pub exit_code: i32,
    pub message: String,
}
fn failure(exit_code: i32, message: impl Into<String>) -> CliFailure {
    CliFailure {
        exit_code,
        message: message.into(),
    }
}

pub fn main_args(args: &[String]) -> Result<(), CliFailure> {
    let (input_path, output_path) = match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        [] | ["--help"] | ["-h"] | ["help"] => {
            print!("{HELP}");
            return Ok(());
        }
        ["validate", "--input", input] => (input.to_string(), None),
        ["run", "--input", input, "--out", output] => (input.to_string(), Some(output.to_string())),
        _ => return Err(failure(2, format!("invalid arguments\n{HELP}"))),
    };
    let bytes = io::read_input(Path::new(&input_path)).map_err(|e| failure(2, e))?;
    let (input, capture) = wire::resolve(wire::decode(&bytes).map_err(|e| failure(2, e))?)
        .map_err(|e| failure(2, e))?;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("survey repository parent");
    source_identity::verify(root, COMPILED_W1_INPUTS_SHA256)
        .map_err(|e| failure(1, format!("current W1 inputs: {e}")))?;
    let executable = std::env::current_exe().map_err(|e| failure(1, e.to_string()))?;
    let binary = binary_sha(&executable).map_err(|e| failure(1, e))?;
    let input_bytes = serde_json::to_vec(&input).map_err(|e| failure(2, e.to_string()))?;
    let identity = input_bytes
        .iter()
        .fold(14_695_981_039_346_656_037_u64, |h, b| {
            (h ^ u64::from(*b)).wrapping_mul(1_099_511_628_211)
        });
    let identity = format!("war1-input-v1:0x{identity:016x}");
    let Some(output_path) = output_path else {
        println!(
            "{}",
            json!({"input":input,"capture":capture,"input_identity":identity,"purpose":"engineering_case","registered":false})
        );
        return Ok(());
    };
    let output = Path::new(&output_path);
    io::create_output_directory(output).map_err(|e| failure(1, e))?;
    let metadata = json!({
        "purpose":"engineering_case", "registered":false,
        "input_sha256":format!("{:x}", Sha256::digest(&bytes)), "input_identity":identity,
        "resolved_input":input, "capture":capture,
        "compiled_inputs_sha256":COMPILED_W1_INPUTS_SHA256,"compiled_input_paths":COMPILED_W1_INPUT_PATHS,
        "source_root":root, "binary_path":executable,"binary_sha256":binary,
        "toolchain":COMPILED_W1_TOOLCHAIN,"build_flags":COMPILED_W1_FLAGS,
        "features":["war-benchmarks"],"input_path":input_path,"output_path":output_path,
    });
    let mut journal = None;
    let mut summary = None;
    let mut semantic_failure = None;
    let execution = (|| -> Result<(), String> {
        io::save_bytes(&output.join("input.json"), &bytes)?;
        io::save_json(&output.join("metadata.json"), &metadata)?;
        journal = Some(Journal::new(&output.join("frames.jsonl"))?);
        let writer = journal.as_mut().expect("created journal");
        match run_to(&input, &capture, &mut |record| writer.append(record)) {
            Ok(result) => summary = Some(result),
            Err(error) => {
                let reason = format!("{}: {}", error.kind, error.detail);
                semantic_failure = Some(error);
                return Err(reason);
            }
        }
        let receipt = writer.finish()?;
        io::sync_directory(output)?;
        save_success(
            output,
            summary.as_ref().expect("successful semantic run"),
            &receipt,
        )?;
        Ok(())
    })();
    if let Err(reason) = execution {
        let receipt = journal
            .as_ref()
            .map(Journal::acknowledged)
            .unwrap_or_else(|| JournalReceipt {
                sha256: format!("{:x}", Sha256::digest([])),
                bytes: 0,
                acknowledged_steps: 0,
            });
        let completed = semantic_failure
            .as_ref()
            .map(|f| f.completed_steps)
            .or_else(|| summary.as_ref().map(|s| s.completed_steps))
            .unwrap_or(0);
        let emitted = semantic_failure
            .as_ref()
            .map(|f| f.emitted_steps)
            .or_else(|| summary.as_ref().map(|s| s.completed_steps))
            .unwrap_or(0);
        let detail = checkpoint_bounded(&semantic_failure);
        let receipt = json!({"status":"failed","purpose":"engineering_case","registered":false,"reason":reason,"completed_steps":completed,"emitted_steps":emitted,"journal":receipt,"semantic_failure":detail});
        let failure_write = io::save_json(&output.join("failure.json"), &receipt)
            .and_then(|()| io::sync_directory(output))
            .and_then(|()| sync_parent(output));
        return Err(failure(
            1,
            match failure_write {
                Ok(()) => reason,
                Err(error) => format!("{reason}; failure receipt also failed: {error}"),
            },
        ));
    }
    println!("saved engineering case to {}", output.display());
    Ok(())
}
fn binary_sha(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut hash = Sha256::new();
    let mut buffer = [0_u8; 65536];
    loop {
        let count = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hash.finalize()))
}
fn checkpoint_bounded(failure: &Option<RunFailure>) -> serde_json::Value {
    match failure {
        Some(failure)
            if failure.checkpoint.as_ref().is_some_and(|c| {
                c.contact_rng.len() > 16 * 1024 || c.casualty_rng.len() > 16 * 1024
            }) =>
        {
            json!({"kind":failure.kind,"detail":failure.detail,"attempted_step":failure.attempted_step,"checkpoint_unavailable":"RNG text exceeds 16 KiB per stream"})
        }
        _ => json!(failure),
    }
}
fn sync_parent(output: &Path) -> Result<(), String> {
    io::sync_directory(
        output
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new(".")),
    )
}
fn save_success(
    output: &Path,
    summary: &RunSummary,
    receipt: &JournalReceipt,
) -> Result<(), String> {
    // Own this marker only after create_new succeeds. Failed final synchronization revokes it.
    let path = output.join("success.json");
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|e| format!("create success marker: {e}"))?;
    let result = (|| {
        let bytes = serde_json::to_vec_pretty(&json!({"status":"complete","purpose":"engineering_case","registered":false,"summary":summary,"journal":receipt})).map_err(|e| e.to_string())?;
        file.write_all(&bytes)
            .and_then(|()| file.write_all(b"\n"))
            .and_then(|()| file.flush())
            .and_then(|()| file.sync_all())
            .map_err(|e| format!("save success marker: {e}"))?;
        io::sync_directory(output)?;
        sync_parent(output)
    })();
    if let Err(reason) = result {
        return Err(
            match std::fs::remove_file(&path).and_then(|()| File::open(output)?.sync_all()) {
                Ok(()) => reason,
                Err(error) => {
                    format!("{reason}; revoke incomplete success marker also failed: {error}")
                }
            },
        );
    }
    Ok(())
}
