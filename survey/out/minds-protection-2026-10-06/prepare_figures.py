"""Render saved registered estimates and complete cell summaries; never simulate.

Usage: python3 prepare_figures.py ANALYSIS_JSON OUTPUT_DIRECTORY
Only registered estimates receive uncertainty intervals. Cell tables show descriptive
arithmetic means and complete observed ranges, without added statistical contrasts.
"""
import collections
import hashlib
import json
import math
import pathlib
import platform
import statistics
import sys

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
import numpy as np
import scipy
from scipy.stats import t


source, out = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
out.mkdir(parents=True, exist_ok=True)
(out / "figures").mkdir(exist_ok=True)
a = json.loads(source.read_text())
conditions = {c["id"]: c for c in a["manifest"]["conditions"]}
rows = collections.defaultdict(list)
lookup = {}
for r in a["endpoints"]:
    rows[r["condition"]].append(r)
    assert (r["condition"], r["seed"]) not in lookup
    lookup[r["condition"], r["seed"]] = r
seeds = list(range(10001, 10041))
assert a["manifest"]["seeds"] == seeds
assert len(conditions) == 192 and len(lookup) == 7680
assert len(a["estimates"]) == 208 and sum(e["primary"] for e in a["estimates"]) == 64
for c, rs in rows.items():
    assert sorted(r["seed"] for r in rs) == seeds
    assert all(r["completed_ticks"] == 64 and not r["lineage_errors"] for r in rs)

# Independent arithmetic audit of each already registered paired estimate.
audit = []
metric = {"original_food_transferred": "thief_transferred", "owner_ticks_alive": "owner_ticks_alive", "discovered_encounters": "discoveries"}
for e in a["estimates"]:
    assert not e["unavailable_seeds"] and e["summary"] is not None
    if e["id"].startswith("opportunity interaction "):
        cid = e["id"].removeprefix("opportunity interaction ")
        terms = [(cid, 1), (cid.replace("p=selective", "p=off"), -1),
                 (cid.replace("r=observed", "r=private"), -1),
                 (cid.replace("p=selective", "p=off").replace("r=observed", "r=private"), 1)]
    else:
        left, right = e["id"].split(" minus ")
        terms = [(left, 1), (right, -1)]
    diffs = [sum(lookup[cid, seed][metric[e["metric"]]] * weight for cid, weight in terms) for seed in seeds]
    mean = statistics.mean(diffs)
    half = float(t.ppf(.975, 39)) * statistics.stdev(diffs) / math.sqrt(40)
    s = e["summary"]
    assert s["n"] == 40 and abs(s["mean"] - mean) < 1e-12
    error = max(abs(s["ci95"][0] - (mean - half)), abs(s["ci95"][1] - (mean + half)))
    assert error < 1e-9, (e["id"], error)
    assert [s["positive"], s["zero"], s["negative"]] == [sum(v > 0 for v in diffs), sum(v == 0 for v in diffs), sum(v < 0 for v in diffs)]
    audit.append({"id": e["id"], "metric": e["metric"], "interval_max_abs_error": error})

cells = []
for cid in sorted(conditions):
    rs = rows[cid]
    scalar = {}
    for key in rs[0]:
        values = [r[key] for r in rs]
        if all(isinstance(v, (float, int, bool)) for v in values):
            scalar[key] = {"mean": statistics.mean(values), "min": min(values), "max": max(values)}
    distributions = {}
    for key in ["sources", "phases", "opportunities", "restriction_ticks", "cancellations", "lineage_errors"]:
        counts = collections.Counter(json.dumps(r[key], sort_keys=True) for r in rs)
        distributions[key] = [{"value": json.loads(k), "episodes": v} for k, v in sorted(counts.items())]
    cells.append({"condition": cid, "panel": conditions[cid]["panel"], "lab": conditions[cid]["lab"], "n": 40,
                  "scalar": scalar, "distributions": distributions,
                  "duplicates": next(d for d in a["duplicates"] if d["condition"] == cid)})
inputs = {"schema": "minds-protection-figures-v1", "analysis_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
          "seeds": seeds, "estimates": a["estimates"], "cells": cells,
          "meaning": "Exact saved estimates; cell means/ranges and categorical counts preserve all 40 episodes per cell. No pooling of cells, additional contrasts or inference."}
(out / "figure-inputs.json").write_text(json.dumps(inputs, indent=2) + "\n")
(out / "reporting-validation.json").write_text(json.dumps({"analysis_sha256": inputs["analysis_sha256"], "conditions": 192,
    "endpoints": 7680, "primary_estimates": 64, "audited_estimates": 208, "audits": audit,
    "versions": {"python": platform.python_version(), "matplotlib": matplotlib.__version__, "numpy": np.__version__, "scipy": scipy.__version__}}, indent=2) + "\n")

plt.rcParams.update({"font.family": "DejaVu Sans", "font.size": 9, "axes.spines.top": False,
                     "axes.spines.right": False, "svg.fonttype": "none", "savefig.dpi": 180})
figure_coverage = {}
def save(fig, name, estimate_keys=None, cell_ids=None):
    for ext in ["png", "svg"]:
        target = out / "figures" / f"{name}.{ext}"
        fig.savefig(target, facecolor="white", bbox_inches="tight")
        if ext == "svg":
            # Matplotlib emits trailing spaces in multiline SVG path attributes.
            target.write_text("\n".join(line.rstrip() for line in target.read_text().splitlines()) + "\n")
    plt.close(fig)
    figure_coverage[name] = {"estimates": estimate_keys or [], "cells": cell_ids or []}

labels = {"original_food_transferred": "Original food transferred (units)", "owner_ticks_alive": "Owner ticks alive", "discovered_encounters": "Discovered encounters"}
def factors(cid, exclude=()):
    return " · ".join(part for part in cid.split("/")[1:] if part.split("=")[0] not in exclude)

# Primary plot: two comparators separately visible within every stratum.
fig, axes = plt.subplots(2, 2, figsize=(12, 9), sharex="col", layout="constrained")
covered = []
for mirrored in [0, 1]:
    for col, met in enumerate(["original_food_transferred", "owner_ticks_alive"]):
        ax = axes[mirrored, col]
        ids = sorted(cid for cid in conditions if cid.startswith("single/p=selective/") and cid.endswith(f"m={mirrored}"))
        for offset, comparator, color, mark in [(-.12, "off", "#2166ac", "o"), (.12, "indiscriminate", "#b35806", "s")]:
            es = [next(e for e in a["estimates"] if e["id"] == cid + " minus " + cid.replace("p=selective", f"p={comparator}") and e["metric"] == met) for cid in ids]
            values = [e["summary"]["mean"] for e in es]
            errors = np.array([[e["summary"]["mean"] - e["summary"]["ci95"][0], e["summary"]["ci95"][1] - e["summary"]["mean"]] for e in es]).T
            ax.errorbar(values, np.arange(len(ids)) + offset, xerr=errors, fmt=mark, color=color, capsize=3, label=f"Selective − {comparator.title()}")
            covered.extend({"id": e["id"], "metric": met} for e in es)
        ax.set_yticks(range(len(ids)), [factors(cid, ("p", "m")) for cid in ids]); ax.invert_yaxis()
        ax.axvline(0, color="#666", lw=.7); ax.grid(axis="x", alpha=.2)
        ax.set_title(f"{'Base' if not mirrored else 'Reflected'} orientation · {labels[met]}")
        ax.set_xlabel("Paired difference: Selective minus comparator")
        ax.legend(loc="best", fontsize=8)
fig.suptitle("Minds P3 · all 64 registered primary estimates\n40 paired seeds per estimate; descriptive Student-t 95% intervals (zero-width when differences agree)")
save(fig, "01-primary", covered)

def contrast_plot(prefix, name, title, metrics):
    selected = [e for e in a["estimates"] if e["id"].startswith(prefix)]
    fig, axes = plt.subplots(2, len(metrics), figsize=(7 * len(metrics), 11 if prefix == "stumble/" else 9), layout="constrained", squeeze=False)
    covered = []
    for m in [0, 1]:
        for col, met in enumerate(metrics):
            es = sorted([e for e in selected if e["metric"] == met and e["id"].split(" minus ")[0].endswith(f"m={m}")], key=lambda e:e["id"])
            ax = axes[m, col]
            for y, e in enumerate(es):
                s = e["summary"]
                ax.errorbar(s["mean"], y, xerr=[[s["mean"] - s["ci95"][0]], [s["ci95"][1] - s["mean"]]], fmt="o", color="#2166ac", capsize=3)
                covered.append({"id": e["id"], "metric": met})
            excluded = ("m", "f") if prefix == "single/p=erased/" else ("m", "r", "f")
            ax.set_yticks(range(len(es)), [factors(e["id"].removeprefix("opportunity interaction ").split(" minus ")[0], excluded) for e in es])
            ax.invert_yaxis(); ax.axvline(0, color="#666", lw=.7); ax.grid(axis="x", alpha=.2)
            ax.set_title(f"{'Base' if not m else 'Reflected'} · {labels[met]}"); ax.set_xlabel("Paired difference (40 seeds)")
    fig.suptitle(title + "\nDescriptive Student-t 95% intervals; each orientation and registered stratum retained")
    save(fig, name, covered)
contrast_plot("opportunity interaction ", "02-opportunity", "Observed − private redeposit opportunity: difference in Selective − Off effects", ["original_food_transferred", "owner_ticks_alive"])
contrast_plot("single/p=erased/", "03-memory-erased", "Memory intervention: Erased − Off", ["original_food_transferred", "owner_ticks_alive"])
contrast_plot("stumble/", "04-stumble-contrasts", "Supplied contact schedule: discovery probability 0.25 − 0", ["original_food_transferred", "owner_ticks_alive", "discovered_encounters"])

# Annotated diagnostic tables: normalized shading within columns, numbers retain units.
diagnostics = [("thief_transferred", "Transferred\nunits"), ("owner_ticks_alive", "Owner alive\nticks"),
    ("attempts", "Attempts"), ("withdrawn", "Withdrawn\nunits"), ("redeposited", "Redeposited\nunits"),
    ("protective_ticks", "Protective\nticks"), ("distance", "Distance\nsteps"), ("burial_cost", "Burial cost\nunits"),
    ("tagged_consumed", "Tagged\nconsumed"), ("tagged_terminal_loss", "Tagged\nloss"), ("sightings", "Sightings"), ("discoveries", "Discoveries")]
for num, panel in enumerate(["single", "mixed", "cue", "stumble"], 5):
    cs = [c for c in cells if c["panel"] == panel]
    metrics = diagnostics.copy()
    if panel == "mixed": metrics.insert(2, ("mixed_selectivity", "Selectivity\nA − B"))
    if panel == "cue": metrics.insert(2, ("cue_errors", "Cue errors"))
    fig, ax = plt.subplots(figsize=(18, max(7, len(cs) * .22 + 2.2)))
    fig.subplots_adjust(left=.30, right=.99, top=.85, bottom=.03)
    data = np.array([[c["scalar"][key]["mean"] for key, _ in metrics] for c in cs])
    scale = np.maximum(np.max(np.abs(data), axis=0), 1)
    ax.imshow(data / scale, cmap="Blues", vmin=0, vmax=1, aspect="auto")
    for row in range(len(cs)):
        for col in range(len(metrics)):
            ax.text(col, row, f"{data[row,col]:.3g}", ha="center", va="center", fontsize=7.5, color="white" if data[row,col]/scale[col] > .65 else "#222")
    ax.set_xticks(range(len(metrics)), [label for _, label in metrics], fontsize=8)
    ax.xaxis.tick_top(); ax.set_yticks(range(len(cs)), [factors(c["condition"]) for c in cs], fontsize=7.5)
    ax.tick_params(length=0); ax.set_title(f"Minds P3 · every {panel} cell (n=40 each)\nArithmetic means in original units; shading scales each column independently", pad=40)
    save(fig, f"{num:02d}-{panel}-cells", cell_ids=[c["condition"] for c in cs])

assert sum(len(v["estimates"]) for v in figure_coverage.values()) == 208
assert sum(len(v["cells"]) for v in figure_coverage.values()) == 192
(out / "figure-coverage.json").write_text(json.dumps(figure_coverage, indent=2) + "\n")
print(json.dumps({"conditions":192,"endpoints":7680,"estimates":208,"primary":64,"figures":len(figure_coverage),"max_interval_audit_error":max(x["interval_max_abs_error"] for x in audit)}))
