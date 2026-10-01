"""Following the Crowd, episode 3, "Variations on a theme": later researchers'
versions of Schelling's board under their own rules, each captioned claim
measured over 20 seeds (see studio/measure.py and
docs/superpowers/specs/2026-09-30-variations-spike.md). The rules follow the
survey's `variations.*` (survey/src/claims/variations.rs), their numbers the
papers'.

- pv-flat, pv-p50, pv-spiked, pv-p100 (Pancs & Vriend 2007, 5 × 5): 5,000
  steps (100,000 turns), their cluster count; pv-ring: 5,000 steps.
- gvn-frozen, gvn-segregated, gvn-mixed (Gauvin et al. 2009): 1,000 steps.
- svw-small, svw-large (Singh et al. 2009): at rest (300 steps).
- zhang-checkerboard (Zhang 2004): 1,000 steps; zhang-random: 20 steps, the
  first at or below 8,000 mixed pairs (his cutoff read as his scaled
  potential; 600 pairs is the least possible).
"""

import concurrent.futures

import measure as m
from measure import median

PV, GVN, SVW, ZHANG = 5000, 1000, 300, 1000


def _series(preset, seed, ticks, tmp):
    rows, _ = m.run(preset, seed, ticks, tmp)
    return rows


def _col(rows, key):
    return [float(r[key]) for r in rows]


def _first(values, test):
    return next((i for i, v in enumerate(values) if test(v)), None)


def seed_row(tmp, seed):
    out = {}
    for name in ("flat", "p50", "p100", "spiked"):
        rows = _series(f"pv-{name}", seed, PV, tmp)
        c = _col(rows, "pv_clusters")
        out[f"{name}_clusters"] = c[-1]
        out[f"{name}_split"] = float(c[-1] == 2)
        out[f"{name}_first_split"] = _first(c, lambda v: v == 2)
    ring = _col(_series("pv-ring", seed, PV, tmp), "groups")
    out["ring_groups"], out["ring_first_two"] = ring[-1], _first(ring, lambda v: v == 2)
    for name in ("frozen", "segregated", "mixed"):
        rows = _series(f"gvn-{name}", seed, GVN, tmp)
        out[f"{name}_s"] = _col(rows, "seg_s")[-1]
        out[f"{name}_still"] = float(_col(rows, "moves")[-1] == 0)
        out[f"{name}_clusters"] = _col(rows, "clusters")[-1]
    for name in ("small", "large"):
        rows = _series(f"svw-{name}", seed, SVW, tmp)
        out[f"{name}_clusters"] = _col(rows, "clusters8")[-1]
        out[f"{name}_rest"] = _first(_col(rows, "moves")[1:], lambda v: v == 0)
    pairs = _col(_series("zhang-checkerboard", seed, ZHANG, tmp), "mixed_pairs")
    out["zhang_start"], out["zhang_100"], out["zhang_end"] = pairs[0], pairs[100], pairs[-1]
    scaled = _col(_series("zhang-random", seed, 20, tmp), "mixed_pairs")
    first = _first(scaled, lambda v: v <= 8000)
    out["zhang_first_8000"] = first if first is not None else 99
    return out


def verdicts(rows, config):
    md = {k: median(rows, k) for k in next(iter(rows.values())) if "first" not in k and "rest" not in k}
    n = len(rows)
    split = {k: sum(r[f"{k}_split"] for r in rows.values()) for k in ("flat", "p50", "p100", "spiked")}
    count = lambda key, test: sum(1 for r in rows.values() if test(r[key]))  # noqa: E731
    mean = lambda key: sum(r[key] for r in rows.values()) / n  # noqa: E731
    return [
        ("Schelling's checkerboard has been rebuilt many times; each rebuild changes one of his rules",
         all(config["changed"].values()),
         "each variation differs from s71-board in its movers, movement, start, edges or neighborhood: "
         + "; ".join(f"{k}: {v}" for k, v in config["changed"].items())),
        ("Pancs and Vriend: anyone may move, any time, to the square they like best",
         config["pv"] == ("anyone", "best"), f"pv-flat moves {config['pv'][0]}, to the {config['pv'][1]} square"),
        ("with Schelling's wishes, most runs end split in two", split["flat"] > n / 2,
         f"{split['flat']:.0f} of {n} end in two clusters (their count; mean {mean('flat_clusters'):.2f}; "
         f"theirs 91 %, 2.10)"),
        ("wanting a mixed street, up to half and half, splits them even more often",
         split["p50"] > split["flat"],
         f"p50: {split['p50']:.0f} of {n} in two clusters against {split['flat']:.0f} (theirs 98 % and 91 %)"),
        ("in a ring, even Flumps who like half and half best end in two groups",
         count("ring_groups", lambda g: g == 2) == n and config["ring"] == "p100",
         f"{count('ring_groups', lambda g: g == 2)} of {n} in two groups at 100,000 turns ({config['ring']})"),
        ("Gauvin and colleagues: anyone may move to a square that suits; tolerance works like temperature",
         config["gvn"] == ("anyone", "random"), f"gvn movers {config['gvn'][0]}, movement {config['gvn'][1]}"),
        ("tolerate too little, and nobody can move",
         count("frozen_still", bool) == n and md["frozen_s"] < 0.1,
         f"at T = 0.3, {count('frozen_still', bool)} of {n} end with nobody moving; median s {md['frozen_s']:.3f}"),
        ("tolerate up to half, and two great clusters form", md["segregated_s"] >= 0.9,
         f"at T = 1/2, median s {md['segregated_s']:.3f}, median clusters {md['segregated_clusters']:.0f}"),
        ("tolerate most, and the town stays mixed", md["mixed_s"] < 0.2,
         f"at T = 0.8, median s {md['mixed_s']:.3f}, median clusters {md['mixed_clusters']:.0f}"),
        ("Singh and colleagues: in a small town, the same wishes make two clusters",
         count("small_clusters", lambda c: c == 2) > n / 2,
         f"8 × 8: {count('small_clusters', lambda c: c == 2)} of {n} end in two clusters (joined at sides or corners)"),
        ("in a city of 100 by 100, dozens; Schelling's striking picture is a small-town effect",
         md["large_clusters"] >= 24,
         f"100 × 100: median {md['large_clusters']:.0f} clusters (theirs 55)"),
        ("Zhang: no empty houses; neighbors trade homes, and everyone likes half and half best",
         config["zhang"] == (10000, 10000, "swap", "tent"),
         f"{config['zhang'][0]} Flumps on {config['zhang'][1]} squares, {config['zhang'][2]}, {config['zhang'][3]} "
         f"utility (best at half alike)"),
        ("from a perfect mix, they sort anyway", md["zhang_end"] < md["zhang_start"] / 4,
         f"mixed pairs {md['zhang_start']:.0f} at the start, {md['zhang_100']:.0f} at step 100, "
         f"{md['zhang_end']:.0f} at step 1,000 (medians)"),
        ("two claims don't reproduce here: Zhang's waiting times, and that wanting only a perfect mix acts like "
         "wanting half and half",
         mean("spiked_clusters") > 1.1 * mean("p100_clusters") and median(rows, "zhang_first_8000") < 2000,
         f"spiked {mean('spiked_clusters'):.2f} clusters against p100's {mean('p100_clusters'):.2f} (means; their "
         f"footnote 23: \"very similar\"); Zhang: 8,000 mixed pairs by step {median(rows, 'zhang_first_8000'):.0f} "
         f"(median), {median(rows, 'zhang_first_8000') * 10000:,.0f} draws, against his 40 million"),
        ("change the rules, and the town still sorts, unless people tolerate a lot, or can't move at all",
         md["segregated_s"] >= 0.9 and md["mixed_s"] < 0.2 and md["frozen_s"] < 0.1 and split["flat"] > n / 2,
         f"s {md['frozen_s']:.2f} frozen, {md['segregated_s']:.2f} at half, {md['mixed_s']:.2f} at 0.8; "
         f"Pancs & Vriend's flat {split['flat']:.0f} of {n} split"),
    ]


def _config(tmp):
    board = m.preset_config("s71-board", tmp)
    keys = ("movers", "movement", "start", "edges", "neighborhood", "radius")
    changed = {}
    for p in ("pv-flat", "gvn-segregated", "svw-small", "zhang-checkerboard"):
        c = m.preset_config(p, tmp)
        changed[p] = ", ".join(f"{k} {c[k]}" for k in keys if c[k] != board[k]) or ""
    pv, gvn, z = m.preset_config("pv-flat", tmp), m.preset_config("gvn-segregated", tmp), m.preset_config(
        "zhang-checkerboard", tmp)
    return {"changed": changed, "pv": (pv["movers"], pv["movement"]), "gvn": (gvn["movers"], gvn["movement"]),
            "ring": m.preset_config("pv-ring", tmp)["utility"],
            "zhang": (z["population"], z["width"] * z["height"], z["movement"], z["utility"])}


def measure(tmp):
    config = _config(tmp)
    for p in ("pv-flat", "pv-p50", "pv-p100", "pv-spiked", "pv-ring", "gvn-frozen", "gvn-segregated", "gvn-mixed",
              "svw-small", "svw-large", "zhang-checkerboard", "zhang-random"):
        m.preset_config(p, tmp)
    with concurrent.futures.ThreadPoolExecutor(8) as pool:
        rows = dict(zip(m.SEEDS, pool.map(lambda s: seed_row(tmp, s), m.SEEDS)))
    keys = [k for k in next(iter(rows.values()))]
    for r in rows.values():
        for k in keys:
            r[k] = float(r[k]) if r[k] is not None else 99999.0
    lines = ["## Variations on Schelling", ""] + m.table(rows, keys)
    lines += ["", "Typical seeds: flat " + str(m.typical_seed(rows, ["flat_clusters", "flat_first_split"]))
              + ", p50 " + str(m.typical_seed(rows, ["p50_clusters", "p50_first_split"]))
              + ", spiked " + str(m.typical_seed(rows, ["spiked_clusters"]))
              + ", ring " + str(m.typical_seed(rows, ["ring_first_two"]))
              + ", frozen " + str(m.typical_seed(rows, ["frozen_s"]))
              + ", segregated " + str(m.typical_seed(rows, ["segregated_s", "segregated_clusters"]))
              + ", mixed " + str(m.typical_seed(rows, ["mixed_s"]))
              + ", small " + str(m.typical_seed(rows, ["small_clusters", "small_rest"]))
              + ", large " + str(m.typical_seed(rows, ["large_clusters", "large_rest"]))
              + ", zhang " + str(m.typical_seed(rows, ["zhang_100", "zhang_end"])) + "."]
    return lines, verdicts(rows, config), {"medians": {k: median(rows, k) for k in keys}, "seeds": len(rows)}
