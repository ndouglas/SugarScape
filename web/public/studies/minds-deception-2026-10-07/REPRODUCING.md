# Reproduce from frozen saved evidence

The campaign archive contains all original raw records and the full unfiltered `analysis.json` (1,052,095,171 bytes). The accepted-evidence archive includes the complete frozen source checkout, prospective packet, accepted protocol and actual native executable. The separate saved-reanalysis archive preserves exact byte-identical `analysis.json` and `results.md`. Download all four archives from [the evidence index](index.html); archive manifests and extraction inventories preserve every retained regular member's SHA256, size and mode.

Verify the downloaded archive hashes by changing into the downloaded `archive/` directory and running `shasum -a 256 -c SHA256SUMS` and `shasum -a 256 -c REPORTING-EMPIRICAL-SUPPLEMENT-SHA256SUMS`. The original checksum file includes the initial manifest and extraction inventories, which must be downloaded beside the archives. Extract each `.tar.gz` into a fresh empty directory with `tar -xzf <archive>`; the archived native executable retains its executable mode. Full source and original paths are retained as provenance; relocated users should supply paths for their own extraction directory.

The reporting/empirical archive includes corrected reporting/exporter sources, tests, pinned requirements and environment/font identities under `scientific-execution/actual-artifacts/` and `metadata/`. Its `CLASSIFICATION.json` distinguishes actual reporting from historical synthetic previews. The direct copies in `reporting-tools/` are byte-identical to reviewed tools; they make no scientific parameter or dependency change.

To inspect or plot already-reviewed saved summaries in a new Python 3.13 environment, install the pinned reporting requirements from `reporting-tools/plotting-requirements-lock.txt`. The exporter can read the complete compact projection without simulation or inference:

```sh
python reporting-tools/plot_p4.py --analysis presentation-analysis.json --out <fresh-figure-directory> --completed-authorized
```

This command copies the 128 native paired summaries and plots them; it does not recalculate paired estimates. A fresh render has its own tool/environment identity and should not be labelled the original accepted render. Exact original PNG/SVG and input identities remain in `figures/plot-inventory.json`. The complete full-analysis streaming path additionally expects its recorded measurement audit beside the output directory, as documented by the unchanged tool source. Historical commands and absolute original locations are preserved in the supplement rather than silently rewritten.

The native registered measurement was launched once. Scientific execution and strict saved-data reanalysis receipts are under `provenance/`; their frozen command/source/binary identities should be read before any new reproduction. A new scientific run would be a separately identified reproduction, never a retry or replacement of these registered records. No new measurement or rendering was performed for this publication packet.
