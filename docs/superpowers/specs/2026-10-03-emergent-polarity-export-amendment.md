# Emergent polarity: strict report export correction

**Date:** 2026-10-03 UTC. Serialization correction after the first complete
registered telemetry run. Every original computation function in `analysis.py`,
including `report` and the Markdown renderer, remains byte-identical. Manifest,
source, authoritative resolved configurations, raw sessions, 100,000 draws,
seed 2026100202, scientific judges and multiplicity rules remain fixed.

The complete original analysis computed for 1,054.856188 seconds, then failed
before emitting either report. Its `episode_end_causes` dictionaries contain
both Python `None` keys for censored episodes and string keys for named end
causes. Sorted JSON attempts to compare those types and raises `TypeError`.
The failed command, runtime and stderr are retained; original source, executable
and resolution hashes were successfully reverified at that boundary. No complete
original verdict artifact survived this export failure.

The tested `export_report.py` serializer retains strict `allow_nan=False` and
uses `sort_keys=False`. Both the documented `analysis.py` CLI and the explicit
`export_report.py` entrypoint call it. No report field or numeric value is
changed. Standard JSON represents a Python `None` dictionary key as the string
**`"null"`**, so that key in `episode_end_causes` names the count of null causes.
It is distinct from the **actual JSON null** `end_cause` value in an individual
censored episode. Censoring remains separately counted; no completed end cause
is invented. Tests exercise both complete entrypoints, mixed null/named counts,
raw null episode values, identical report values/verdicts, and nonfinite rejection.

The complete original report is recomputed with the same registered inputs and
rules and retained before final corrected analysis. Its original computation
function source and hashes, immutable input hashes, original failed invocation
and successful export invocation bind this repair in the execution provenance.
The independently diagnosed sequential engine correction may proceed during that
recomputation under the controller's ruling, because this analysis depends only
on preserved immutable files and computation functions. Final scientific results
use the separately documented complete assembled dataset.
