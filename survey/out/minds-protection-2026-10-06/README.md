# Minds P3 reporting packet — campaign 2026-10-06

The fixed native campaign contains all 192 cells × 40 seeds (7,680 episodes), with all 208 registered descriptive estimates. The campaign label uses America/New_York; actual execution began on 2026-10-07 UTC. **Reporting and publication review is pending.** The historical registration keeps its original bytes/status; the [dated findings](../../../docs/superpowers/specs/2026-10-06-minds-protection-findings.md) records later execution.

- [Findings](../../../docs/superpowers/specs/2026-10-06-minds-protection-findings.md) explains conditional food/lifetime results, costs and limitations.
- [analysis.json](analysis.json) is the exact complete scientific analysis: 7,680 endpoints, all 208 estimates and 192 duplicate diagnostics.
- [results.md](results.md) is the exact generated report with all comparisons, cells and source/opportunity evidence.
- [index.json](index.json) retains original campaign raw paths and source/registration revisions. [index.incomplete.json](index.incomplete.json) preserves the initial pre-completion record; it is not a failed scientific attempt.
- [figure-inputs.json](figure-inputs.json) retains all saved estimates and all cell means/ranges/categorical counts; [figure-coverage.json](figure-coverage.json) maps every plotted estimate and cell.
- [prepare_figures.py](prepare_figures.py), [plotting-command.json](plotting-command.json) and [reporting-validation.json](reporting-validation.json) retain rendering versions/command and the independent arithmetic audit of all estimates.
- [provenance.json](provenance.json) and [public member hashes](public-members.json) identify this reporting packet. The [actual prospective gate](provenance/task-p3-prospective-review.md), [readiness report](provenance/task-p3-readiness-report.md), [campaign report](provenance/task-p3-campaign-report.md), [raw member inventory](provenance/campaign-raw-member-inventory.json), [campaign census](provenance/campaign-census-validation.json), command/start/exit/seal and reanalysis receipts are retained under `provenance/`.

| Figure | Complete coverage | SVG |
|---|---|---|
| [Primary outcomes](figures/01-primary.png) | 64 registered estimates, both orientations | [SVG](figures/01-primary.svg) |
| [Redeposit opportunity](figures/02-opportunity.png) | 16 interactions | [SVG](figures/02-opportunity.svg) |
| [Memory intervention](figures/03-memory-erased.png) | 32 Erased − Off estimates | [SVG](figures/03-memory-erased.svg) |
| [Stumble encounters](figures/04-stumble-contrasts.png) | 96 registered contrasts | [SVG](figures/04-stumble-contrasts.svg) |
| [Single cells](figures/05-single-cells.png) | All 64 cells | [SVG](figures/05-single-cells.svg) |
| [Mixed cells](figures/06-mixed-cells.png) | All 32 cells | [SVG](figures/06-mixed-cells.svg) |
| [Cue cells](figures/07-cue-cells.png) | All 32 cells | [SVG](figures/07-cue-cells.svg) |
| [Stumble cells](figures/08-stumble-cells.png) | All 64 cells | [SVG](figures/08-stumble-cells.svg) |

Re-render without simulation:

```sh
python3 survey/out/minds-protection-2026-10-06/prepare_figures.py \
  survey/out/minds-protection-2026-10-06/analysis.json /tmp/minds-p3-figures
```

The script reads saved data only. Registered paired intervals are plotted unchanged; cell tables use descriptive arithmetic means. Shading scales each diagnostic column independently and never implies a common unit or combined score. Figure labels use `i` initial visibility, `r` later redeposit opportunity, `c` reburial cost, `m` orientation (0 base/1 reflected), `o` deposition order, `e` cue-control type and `f` discovery probability; `p` names the policy.

PNG figures preserve the rendered DejaVu Sans appearance. SVG retains editable text and references DejaVu Sans; viewers without that font may substitute a local font. Independent Chrome rendering of all eight SVGs found the labels/axes readable without clipping after the label/margin correction; the formal review record will be retained with publication evidence.

The external scientific archive retains all 571,055,314 raw bytes, complete tracked source and the frozen executable; raw files are not embedded in this smaller public packet. Relative `raw/` references in the unchanged index resolve in the external full campaign directory, not here. The verified science baseline has been staged at `/Users/nathan/.local/share/sugarscape/evidence/minds-protection-2026-10-06-staging-20261007T014345375561Z`; final report/review/publication archival remains incomplete until the controller adds its final receipt. Historical WASM/Vitest receipts remain unavailable as explicitly recorded in the prospective review; no fresh WASM execution is claimed.

This is SugarScape's own mechanism experiment. No orientation pooling, favorable filtering, overall Holds/Fails, animal-cognition, deception, source-paper numerical replication or safety conclusion is assigned.
