"""The Contagion episode's captioned claims, measured over 20 seeds (see
studio/measure.py and docs/superpowers/specs/2026-09-27-contagion-episode.md):

- v-1-rid (10 diseases, 4 each) and v-2-endemic (25 diseases, 10 each) over
  1000 ticks.
- v-mcneill: a society that has learned away its diseases meets a novel
  10-bit one (the longest the book's diseases can be) at tick 300, given to
  5 Flumps; against the same seeds without the outbreak.
- Our experiment, labeled as one: the same outbreak with a 40-bit disease.
"""

import statistics

import measure as m
from measure import median

TICKS = 1000
OUTBREAK = 300
PLAGUE_END = 500
LONG = 40  # bits: four times the book's longest disease


def _count(rows, test):
    return sum(bool(test(r)) for r in rows.values())


def _sick(frame):
    n = len(frame.diseases)
    return sum(1 for v in frame.diseases.values() if v) / n if n else float("nan")


def verdicts(rows):
    """Each captioned claim, whether the 20 seeds support it, and why. Rule E
    follows the book's note 16 (one immune flip per agent per tick) and its
    worked example (a learned disease passed on through its tick); under the
    engine's earlier reading, V-1 kept a residue and V-2 an endemic level."""
    n = len(rows)
    wells = [r["v1_well_by"] for r in rows.values()]
    rid = _count(rows, lambda r: r["v1_end"] == 0)
    cleared = _count(rows, lambda r: r["v2_late"] == 0)
    lower = _count(rows, lambda r: r["ratio"] < 1)
    fewer = _count(rows, lambda r: r["plague_ratio"] < 1)
    return [
        ("at the start, almost every Flump is sick", median(rows, "v1_start") >= 0.85,
         f"sick at tick 0: {median(rows, 'v1_start'):.0%} (median; {min(r['v1_start'] for r in rows.values()):.0%}–"
         f"{max(r['v1_start'] for r in rows.values()):.0%}); agents already immune to a disease are not given it"),
        ("within a few ticks, nearly all are well", max(wells) <= 15,
         f"fewer than 5% sick by tick {statistics.median(wells):g} (median; {min(wells)}–{max(wells)})"),
        ("the book says society rids itself of every disease; it does, in 20 of 20 worlds", rid == n,
         f"no sick Flump at tick {TICKS} in {rid} of {n}"),
        ("give them more diseases than an immune system can hold: the book says disease stays; it clears here too, "
         "in 20 of 20", cleared == n,
         f"V-2 (25 diseases, 10 each): no sick Flump over ticks 500–{TICKS} in {cleared} of {n}"),
        ("the book expects a plague; it spreads to about half the Flumps, and costs almost nothing",
         0.4 <= median(rows, "reach") <= 0.7 and 0.97 <= median(rows, "ratio") <= 1.03 and 6 <= lower <= 14,
         f"Flumps alive at tick {OUTBREAK} that catch the novel 10-bit disease from another Flump by tick {PLAGUE_END}: "
         f"{median(rows, 'reach'):.0%} (median; {min(r['reach'] for r in rows.values()):.0%}–"
         f"{max(r['reach'] for r in rows.values()):.0%}); population at {PLAGUE_END} against no outbreak: "
         f"{median(rows, 'ratio'):.2f}, lower in {lower} of {n} (a coin flip)"),
        ("only a disease four times longer than any in the book sweeps through, and costs about a quarter of the Flumps",
         median(rows, "plague_reach") >= 0.9 and fewer == n and 0.7 <= median(rows, "plague_ratio") <= 0.8,
         f"a {LONG}-bit outbreak: {median(rows, 'plague_reach'):.0%} of the Flumps alive at tick 400 have caught it "
         f"(median); population at {PLAGUE_END} against no outbreak {median(rows, 'plague_ratio'):.2f}, lower in "
         f"{fewer} of {n}"),
    ]


def _outbreak(tmp, seed, length, name):
    """(share of the Flumps alive at the outbreak that catch the novel disease
    by PLAGUE_END, share of those alive at tick 400 that ever caught it, the
    population at PLAGUE_END)."""
    spec = {"preset": "v-mcneill", "ticks": PLAGUE_END, "seed": seed}
    if length != 10:
        spec["set"] = {"disease.outbreaks": [{"tick": OUTBREAK, "agents": 5, "length": {"min": length, "max": length}}]}
    d = m.shot(spec, tmp, name)
    seeded = [i for f in d.frames[OUTBREAK - 1 : OUTBREAK + 2] for i in f.infections if i.infector is None]
    novel, given = {i.disease for i in seeded}, {i.infected for i in seeded}
    # Caught from another Flump: the outbreak's own recipients are left out
    # of both the count and the population it's a share of.
    caught = {i.infected for f in d.frames[OUTBREAK:] for i in f.infections
              if i.disease in novel and i.infector is not None} - given
    at_outbreak, at_400 = set(d.frames[OUTBREAK].agents) - given, set(d.frames[400].agents) - given
    result = (len(caught & at_outbreak) / len(at_outbreak), len(caught & at_400) / len(at_400),
              len(d.frames[PLAGUE_END].agents))
    (tmp / f"{name}.frames.json").unlink()
    return result


def seed_row(tmp, seed):
    v1 = m.shot({"preset": "v-1-rid", "ticks": TICKS, "seed": seed}, tmp, f"v1-{seed}")
    s1 = [_sick(f) for f in v1.frames]
    (tmp / f"v1-{seed}.frames.json").unlink()
    v2 = m.shot({"preset": "v-2-endemic", "ticks": TICKS, "seed": seed}, tmp, f"v2-{seed}")
    s2 = [_sick(f) for f in v2.frames]
    (tmp / f"v2-{seed}.frames.json").unlink()
    control, _ = m.run("v-mcneill", seed, PLAGUE_END, tmp, {"disease.outbreaks": []})
    calm = float(control[PLAGUE_END]["population"])
    reach, _, pop = _outbreak(tmp, seed, 10, f"mc-{seed}")
    _, plague_reach, plague_pop = _outbreak(tmp, seed, LONG, f"plague-{seed}")
    return {
        "v1_start": s1[0],
        "v1_well_by": next((k for k, v in enumerate(s1) if v < 0.05), TICKS),
        "v1_late": statistics.fmean(s1[500:]),
        "v1_end": s1[TICKS],
        "v2_late": statistics.fmean(s2[500:]),
        "reach": reach,
        "ratio": pop / calm,
        "plague_reach": plague_reach,
        "plague_ratio": plague_pop / calm,
        "pop_calm": calm,
        "pop_plague": float(plague_pop),
    }


def measure(tmp):
    rows = {seed: seed_row(tmp, seed) for seed in m.SEEDS}
    keys = ["v1_start", "v1_well_by", "v1_late", "v1_end", "v2_late", "reach", "ratio", "plague_reach",
            "plague_ratio"]
    lines = [f"## v-1-rid and v-2-endemic ({TICKS} ticks); v-mcneill's outbreak, and a {LONG}-bit one", ""]
    lines += m.table(rows, keys)
    lines += ["", f"Typical seed (V-1): {m.typical_seed(rows, ['v1_well_by', 'v1_late'])}; "
              f"(McNeill): {m.typical_seed(rows, ['reach', 'plague_ratio'])}.",
              "Sick: carrying at least one disease."]
    medians = {k: median(rows, k) for k in keys}
    medians.update(pop_calm=median(rows, "pop_calm"), pop_plague=median(rows, "pop_plague"))
    return lines, verdicts(rows), {"medians": medians, "seeds": len(rows)}
