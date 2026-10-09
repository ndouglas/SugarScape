# Lossless scientific archive

Built locally and fully verified on 2026-10-09; independent integration review and remote publication verification remain pending. Scientific revision: `e67b8d293ffb21c3a6fb94daf727e69b7a7af51b`. The [manifest](archive-manifest.json) binds the concatenated archive and every ordered part; the [member manifest](scientific-members.json) maps every original source path to its exact archive name, hash, size and permission bits.

The archive contains **14,934 regular files / 13,067,756,638 logical bytes**, including all **11,523 registered raw files / 12,064,047,212 bytes**. The compressed archive is **277,896,521 bytes** and is split into 6 parts, each at most 48 MiB. All parts are required.

- [scientific-e67b8d2.tar.zst.part-0000](scientific-e67b8d2.tar.zst.part-0000) — 50,331,648 bytes, SHA256 `1f19738edd529e891bd564753a985fed641497313c369a0e0fd742a3e721d0fb`
- [scientific-e67b8d2.tar.zst.part-0001](scientific-e67b8d2.tar.zst.part-0001) — 50,331,648 bytes, SHA256 `52deb1e2be2b981ffe383f927399f3b4946b733e1c125b80202af6adba1e9980`
- [scientific-e67b8d2.tar.zst.part-0002](scientific-e67b8d2.tar.zst.part-0002) — 50,331,648 bytes, SHA256 `c4191fbcae5e5f3769030e2087bd6b04a1b687b85c00d4f23d1824864e78c518`
- [scientific-e67b8d2.tar.zst.part-0003](scientific-e67b8d2.tar.zst.part-0003) — 50,331,648 bytes, SHA256 `76267a220db6de2445b692bd31c27288d4d02bf6b70455acc06dc8e7c33a5bfc`
- [scientific-e67b8d2.tar.zst.part-0004](scientific-e67b8d2.tar.zst.part-0004) — 50,331,648 bytes, SHA256 `ac60c799e100eb493908f579ccfa6c2cb6220d0437192d1f2534861b8adadb71`
- [scientific-e67b8d2.tar.zst.part-0005](scientific-e67b8d2.tar.zst.part-0005) — 26,238,281 bytes, SHA256 `8015f9a96eb5ec9313add7ff5b876491dca4ab9fd7e053d57c4bc4f3d432b36f`

With all files downloaded into one directory and `zstd` available, concatenate in numbered order and extract into a new directory:

```bash
cat scientific-e67b8d2.tar.zst.part-* > scientific-e67b8d2.tar.zst
shasum -a 256 scientific-e67b8d2.tar.zst
mkdir scientific-e67b8d2-extracted
zstd -d -c scientific-e67b8d2.tar.zst | tar -xpf - -C scientific-e67b8d2-extracted
```

Expected whole SHA256: `6b01bed58654d9498578788954a4e9091ce99429b01a82440d3f23712612cd60`. Verify parts and the whole archive against the manifest before extraction. The complete [roundtrip receipt](roundtrip.json) records actual decompression of every member and verifies all hashes, sizes and modes. The [compression receipt](compression.json) records the actual tool/version, deterministic sorted tar metadata and command. Numeric tar uid/gid and mtime are normalized to zero; original regular-file permission bits are preserved. The [original raw inventory comparison](raw-original-identity-comparison.json) checks every registered file against the accepted inventory.

The payload cutoff includes the complete frozen tracked source and accepted-HEAD Git ancestry bundle; actual native executable and selected-input stamp; safe actual build configuration; all 192 corrected construction records, opportunity streams and task oracle; every registered raw start/frame/outcome/index/census file; both complete native saved analyses; the original empirically reviewed report, all 16 PNG/16 SVG assets and all 256 input rows; scanned acceptance/provenance; and scientific allowlisted operational derivatives. Original reporting status text inside `empirically-reviewed-report/` is the unchanged historical candidate snapshot. The current accepted status appears in the surrounding [findings](../findings.md).

Full private inherited environments, credential-bearing operational originals, unrelated Git refs/config/hooks and the full failed operational history remain privately preserved. `metadata/public-derivative-lineage.json` binds private-original and public-derivative identities and exact field-selection rules; no public equality with unredacted private operational originals is claimed. All public logical members and all decoded Git objects reachable from the accepted revision were scanned for eight known inherited sensitive values before compression, with zero matches. This exact-value check does not establish absence of unknown or encoded secrets. Later archive/publication/CI/Pages/served receipts are outside the payload cutoff to avoid self-referential hashes.

`metadata/scientific-source.bundle` contains only the accepted revision and its reachable ancestors under `execution-candidate`. It can restore that exact Git state into a **new** destination with `git clone --branch execution-candidate metadata/scientific-source.bundle new-source`. The tracked `source/` tree separately retains every accepted byte and mode. The native binary is the actual original macOS aarch64 executable, not an integration rebuild. Its saved-data loader retains absolute original source-path and clean Git-revision requirements. Inspecting/extracting this archive is portable; relocating it does not establish a portable strict-loader execution. Restore exact paths only in a dedicated environment without overwriting existing evidence. No collection or native reanalysis was performed during packaging.
