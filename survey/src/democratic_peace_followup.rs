//! Follow-up envelope rules, separate from the schema 1 scientific registration.
use super::*;
pub(super) const SPEC: &str =
    "docs/superpowers/specs/2026-10-04-democratic-peace-precision-design.md";
pub(super) const PROTOCOL: &str = "democratic-peace-followup-v1";
pub(super) fn phase(m: &Value) -> Result<(&str, u64, u64, &str), String> {
    require(
        text(m, "study_protocol")? == PROTOCOL,
        "unknown follow-up protocol",
    )?;
    let result = match text(m, "phase")? {
        "literal_precision" => (
            "printed_decreasing",
            390000001,
            2026100302,
            REGISTERED_ANALYSIS_JOBS_SHA256,
        ),
        "prose_precision" => (
            "prose_increasing",
            410000001,
            2026100402,
            "42c9dabf3a91b43ba1925845959a3ba024b9a314eb68c7a93470f2c8a15cf20f",
        ),
        _ => return Err("unsupported follow-up phase".into()),
    };
    require(
        text(m, "reading")? == result.0,
        "follow-up phase/reading mismatch",
    )?;
    require(
        text(m, "execution_mode")? == "registered"
            && m["precision_registered"] == json!(true)
            && text(m, "runtime_gate")? == "followup_full_precision",
        "follow-up population gate changed",
    )?;
    let h = &m["historical_input"];
    exact(
        h,
        &[
            "study_id",
            "manifest_sha256",
            "source_inventory_sha256",
            "binary_sha256",
            "build_receipt_sha256",
            "resolved_configs_sha256",
            "raw_sha256",
            "preservation_inventory_sha256",
        ],
    )?;
    require(
        text(h, "study_id")? == "democratic-peace-original-2026-10-04",
        "wrong historical study",
    )?;
    for k in [
        "manifest_sha256",
        "source_inventory_sha256",
        "binary_sha256",
        "build_receipt_sha256",
        "resolved_configs_sha256",
        "raw_sha256",
        "preservation_inventory_sha256",
    ] {
        require(is_hash(text(h, k)?), "incomplete historical binding")?;
    }
    Ok(result)
}

pub(super) fn historical(m: &Value) -> Result<(), String> {
    let expected = json!({"study_id":"democratic-peace-original-2026-10-04",
        "manifest_sha256":"8a2f365365ae1b5b5f71d8319225c6e565fb60ffe52e7d1008e5e04f296bb10a",
        "source_inventory_sha256":"fad57148ae9ca5f29d99df7777c235bd03b44187d9254d9d45d7dbd01bd856e2",
        "binary_sha256":"8bfac59be856cd68cf22186eff69c2f070026622fc7f81cfc2d0520f031461a5",
        "build_receipt_sha256":"239c1b10ac022564d9c2a86f7cbe13f63c17c24e47f548942124a170fe86f965",
        "resolved_configs_sha256":"ffb684d35cc0bfe0630c63aeef1e53306a0bdccdb0133a9ae08090338a228455",
        "raw_sha256":"753a1d5ff6236ba4720e0c52e46ab061643a5d27ef5c5d00175e0ff3f6d6c492",
        "preservation_inventory_sha256":"18da5bc191a00f1ed199dc1b54639a925f733b63db5c677a756e3c9bb30215bd"});
    require(
        m["historical_input"] == expected,
        "wrong historical archive binding",
    )
}
fn option_path(opts: &BTreeMap<String, String>, key: &str) -> Result<PathBuf, String> {
    Ok(PathBuf::from(opts.get(key).ok_or_else(|| {
        format!("{key} required before registered follow-up")
    })?))
}
fn review(v: &Value, declaration_sha: &str, runtime_sha: &str) -> Result<(), String> {
    exact(
        v,
        &[
            "schema_version",
            "study_protocol",
            "decision",
            "independent",
            "reviewer",
            "registered_histories",
            "declaration_sha256",
            "runtime_receipt_sha256",
        ],
    )?;
    require(
        uint(v, "schema_version")? == 1
            && text(v, "study_protocol")? == PROTOCOL
            && text(v, "decision")? == "accepted_prospective_followup"
            && v["independent"] == json!(true)
            && !text(v, "reviewer")?.trim().is_empty()
            && uint(v, "registered_histories")? == 0,
        "independent premeasurement declaration review required",
    )?;
    require(
        text(v, "declaration_sha256")? == declaration_sha
            && text(v, "runtime_receipt_sha256")? == runtime_sha,
        "review receipt identity mismatch",
    )
}
fn forecast(v: &Value) -> Result<f64, String> {
    exact(
        v,
        &[
            "cpu_seconds",
            "cpu_hours",
            "generation_cpu_seconds",
            "analysis_cpu_seconds",
            "analysis_writer_cpu_seconds",
            "forecast_multiplier",
            "native_attempts",
            "analysis_jobs",
            "draws_per_job",
            "native_workers",
            "peak_rss_bytes",
            "within_budget",
            "assumptions",
        ],
    )?;
    require(
        uint(v, "forecast_multiplier")? == 2
            && uint(v, "native_attempts")? == 10800
            && uint(v, "analysis_jobs")? == 129
            && uint(v, "draws_per_job")? == 100000
            && uint(v, "native_workers")? == 1
            && v["within_budget"] == json!(true)
            && uint(v, "peak_rss_bytes")? > 0
            && uint(v, "peak_rss_bytes")? <= 2147483648,
        "complete phase forecast exceeds scheduling gate",
    )?;
    let cpu = number(v, "cpu_seconds")?;
    let parts = [
        "generation_cpu_seconds",
        "analysis_cpu_seconds",
        "analysis_writer_cpu_seconds",
    ];
    let mut sum = 0.;
    for k in parts {
        let value = number(v, k)?;
        require(value > 0., "invalid forecast measurement")?;
        sum += value;
    }
    require(
        cpu > 0.
            && cpu <= 43200.
            && (cpu - 2. * sum).abs() <= 1e-8
            && number(v, "cpu_hours")? == cpu / 3600.,
        "forecast arithmetic mismatch",
    )?;
    require(
        array(v, "assumptions")?.iter().all(Value::is_string),
        "forecast assumptions required",
    )?;
    Ok(cpu)
}
fn git(root: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let out = std::process::Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|e| e.to_string())?;
    require(
        out.status.success(),
        "committed source/declaration lookup failed",
    )?;
    Ok(out.stdout)
}
pub(super) fn activation(
    p: &Prepared,
    receipt: &Option<String>,
    root: &Path,
    opts: &BTreeMap<String, String>,
) -> Result<Vec<PathBuf>, String> {
    let dp = option_path(opts, "--declaration")?;
    let rp = option_path(opts, "--declaration-review")?;
    let tp = option_path(opts, "--runtime-receipt")?;
    let db = read(&dp)?;
    let rb = read(&rp)?;
    let tb = read(&tp)?;
    let d = strict_json(&db)?;
    let r = strict_json(&rb)?;
    let t = strict_json(&tb)?;
    exact(
        &d,
        &[
            "schema_version",
            "study_protocol",
            "spec",
            "status",
            "motivated_by_observed_original",
            "historical_input",
            "method_contract_sha256",
            "source_table_sha256",
            "benchmark_evidence_sha256",
            "phase_order",
            "registered_attempts",
            "phases",
        ],
    )?;
    require(
        uint(&d, "schema_version")? == 1
            && text(&d, "study_protocol")? == PROTOCOL
            && text(&d, "spec")? == SPEC
            && text(&d, "status")? == "prospective_frozen_not_blind"
            && d["motivated_by_observed_original"] == json!(true)
            && d["historical_input"] == p.manifest["historical_input"]
            && d["method_contract_sha256"] == p.manifest["method_contract_sha256"]
            && d["source_table_sha256"] == p.manifest["source_table_sha256"]
            && d["phase_order"] == json!(["literal_precision", "prose_precision"])
            && uint(&d, "registered_attempts")? == 21600
            && is_hash(text(&d, "benchmark_evidence_sha256")?),
        "prospective declaration contract mismatch",
    )?;
    let phases = array(&d, "phases")?;
    require(phases.len() == 2, "both phases must freeze beforeL")?;
    for (i, phase) in phases.iter().enumerate() {
        exact(
            phase,
            &[
                "phase",
                "reading",
                "manifest_sha256",
                "binary_sha256",
                "source_inventory_sha256",
                "build_receipt_sha256",
                "resolved_configs_sha256",
                "analysis_jobs_sha256",
                "native_keys_sha256",
            ],
        )?;
        let name = if i == 0 {
            "literal_precision"
        } else {
            "prose_precision"
        };
        require(
            text(phase, "phase")? == name
                && text(phase, "reading")?
                    == if i == 0 {
                        "printed_decreasing"
                    } else {
                        "prose_increasing"
                    },
            "phase declaration order/reading drift",
        )?;
        for key in [
            "manifest_sha256",
            "binary_sha256",
            "source_inventory_sha256",
            "build_receipt_sha256",
            "resolved_configs_sha256",
            "analysis_jobs_sha256",
            "native_keys_sha256",
        ] {
            require(is_hash(text(phase, key)?), "incomplete declaration binding")?;
        }
        let expected_jobs = if i == 0 {
            REGISTERED_ANALYSIS_JOBS_SHA256
        } else {
            "42c9dabf3a91b43ba1925845959a3ba024b9a314eb68c7a93470f2c8a15cf20f"
        };
        require(
            text(phase, "analysis_jobs_sha256")? == expected_jobs,
            "declaration phase job drift",
        )?;
        let mut keys = Vec::new();
        for a in &p.arms {
            for repeat in 0..100 {
                let suffix = a.id.split_once('.').ok_or("bad phase arm")?.1;
                keys.push(json!([
                    format!("{name}.{suffix}"),
                    if i == 0 { 390000001 } else { 410000001 } + a.canonical * 10000 + repeat
                ]));
            }
        }
        require(
            text(phase, "native_keys_sha256")? == hash_json(&json!(keys))?,
            "declaration phase key drift",
        )?;
        if name == text(&p.manifest, "phase")? {
            for (key, value) in object(&identity(p, receipt))? {
                require(
                    phase[key] == *value,
                    "activation native phase binding mismatch",
                )?;
            }
        }
    }
    exact(
        &t,
        &[
            "schema_version",
            "study_protocol",
            "status",
            "declaration_sha256",
            "benchmark_evidence_sha256",
            "phases",
            "combined_cpu_seconds",
        ],
    )?;
    require(
        uint(&t, "schema_version")? == 1
            && text(&t, "study_protocol")? == PROTOCOL
            && text(&t, "status")? == "approved_complete_phase_forecast"
            && text(&t, "declaration_sha256")? == digest(&db)
            && t["benchmark_evidence_sha256"] == d["benchmark_evidence_sha256"],
        "runtime attestation chain mismatch",
    )?;
    exact(&t["phases"], &["literal_precision", "prose_precision"])?;
    let total =
        forecast(&t["phases"]["literal_precision"])? + forecast(&t["phases"]["prose_precision"])?;
    require(
        total <= 86400. && number(&t, "combined_cpu_seconds")? == total,
        "combined24CPUhour forecast gate exceeded",
    )?;
    review(&r, &digest(&db), &digest(&tb))?;
    require(
        git(root, &["status", "--porcelain", "--untracked-files=no"])?.is_empty(),
        "registered follow-up requires clean committed sources",
    )?;
    let committed_path = fs::canonicalize(&dp).map_err(|e| e.to_string())?;
    let relative = committed_path
        .strip_prefix(root)
        .map_err(|_| "declaration must be committed inside source repository")?
        .to_str()
        .ok_or("invalid declaration path")?;
    for entry in array(&p.manifest, "source_inventory")? {
        let name = text(entry, "path")?;
        require(name != relative, "declaration cannot enter own inventory")?;
        require(
            digest(&git(root, &["show", &format!("HEAD:{name}")])?) == text(entry, "sha256")?,
            "normative source not committed atHEAD",
        )?;
    }
    require(
        git(root, &["show", &format!("HEAD:{relative}")])? == db,
        "declaration bytes not committed atHEAD",
    )?;
    let hp = option_path(opts, "--historical-study-root")?;
    let ip = option_path(opts, "--historical-inventory")?;
    let ap = option_path(opts, "--historical-source-archive")?;
    let bp = option_path(opts, "--historical-binary")?;
    let ib = read(&ip)?;
    require(
        digest(&ib)
            == text(
                &p.manifest["historical_input"],
                "preservation_inventory_sha256",
            )?,
        "wrong original preservation inventory",
    )?;
    let entries = strict_json(&ib)?;
    let entries = entries
        .as_array()
        .ok_or("historical inventory must be array")?;
    require(entries.len() == 13, "original13files required")?;
    let mut inputs = vec![dp, rp, tp, ip, ap.clone(), bp.clone()];
    let mut prior = "";
    for entry in entries {
        exact(entry, &["path", "bytes", "sha256"])?;
        let name = text(entry, "path")?;
        require(name > prior, "historical inventory order/duplicate")?;
        prior = name;
        let path = source_path(&hp, Path::new(name))?;
        let bytes = read(&path)?;
        require(
            bytes.len() as u64 == uint(entry, "bytes")? && digest(&bytes) == text(entry, "sha256")?,
            "historical preservation mismatch",
        )?;
        inputs.push(path);
    }
    require(
        digest(&read(&ap)?) == "69f7bbf5acd3a61190eea49df0657c3e72694d987c3de43f109f9bd70b3c70fa"
            && digest(&read(&bp)?) == text(&p.manifest["historical_input"], "binary_sha256")?,
        "historical tar/executable mismatch",
    )?;
    if text(&p.manifest, "phase")? == "prose_precision" {
        let lp = option_path(opts, "--literal-sessions")?;
        let bytes = read(&lp)?;
        require(
            bytes.last() == Some(&b'\n'),
            "literal phase interrupted before prose",
        )?;
        let literal = &phases[0];
        let mut lm = p.manifest.clone();
        lm["phase"] = json!("literal_precision");
        let mut la = Vec::new();
        for a in &p.arms {
            let mut c = serde_json::to_value(&a.config).map_err(|e| e.to_string())?;
            c["probability_direction"] = json!("printed_decreasing");
            la.push(Arm {
                id: format!(
                    "literal_precision.{}",
                    a.id.split_once('.').ok_or("bad phase arm")?.1
                ),
                index: a.index,
                canonical: a.canonical,
                family: a.family.clone(),
                sessions: 100,
                first_seed: 390000001 + a.canonical * 10000,
                config: serde_json::from_value(c).map_err(|e| e.to_string())?,
            });
        }
        let lp_prepared = Prepared {
            manifest: lm,
            arms: la,
            payload: Vec::new(),
            payload_sha: text(literal, "resolved_configs_sha256")?.into(),
            manifest_sha: text(literal, "manifest_sha256")?.into(),
            binary_sha: text(literal, "binary_sha256")?.into(),
            inventory_sha: Some(text(literal, "source_inventory_sha256")?.into()),
        };
        let lr = Some(text(literal, "build_receipt_sha256")?.to_string());
        let mut keys = BTreeSet::new();
        for line in BufReader::new(bytes.as_slice()).lines() {
            let row = strict_json(line.map_err(|e| e.to_string())?.as_bytes())?;
            require(
                keys.insert(validate_existing(&row, &lp_prepared, &lr)?),
                "duplicate literal predecessor key",
            )?;
        }
        require(
            keys.len() == 10800,
            "prose requires full literal attempted census",
        )?;
        inputs.push(lp);
    }
    Ok(inputs)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn independent_review_identity_and_resource_gates_are_strict() {
        let mut r = json!({"schema_version":1,"study_protocol":PROTOCOL,"decision":"accepted_prospective_followup",
            "independent":true,"reviewer":"synthetic-boundary-test","registered_histories":0,
            "declaration_sha256":"a".repeat(64),"runtime_receipt_sha256":"b".repeat(64)});
        assert!(review(&r, &"a".repeat(64), &"b".repeat(64)).is_ok());
        r["independent"] = json!(false);
        assert!(review(&r, &"a".repeat(64), &"b".repeat(64)).is_err());
        let mut f = json!({"cpu_seconds":6.,"cpu_hours":6./3600.,"generation_cpu_seconds":1.,"analysis_cpu_seconds":1.,
            "analysis_writer_cpu_seconds":1.,"forecast_multiplier":2,"native_attempts":10800,"analysis_jobs":129,
            "draws_per_job":100000,"native_workers":1,"peak_rss_bytes":1024,"within_budget":true,"assumptions":["synthetic"]});
        assert_eq!(forecast(&f).unwrap(), 6.);
        f["forecast_multiplier"] = json!(1);
        assert!(forecast(&f).is_err());
        f["forecast_multiplier"] = json!(2);
        f["peak_rss_bytes"] = json!(2147483649u64);
        assert!(forecast(&f).is_err());
    }
}
