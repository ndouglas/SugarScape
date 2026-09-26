"""The Pollution episode's captioned claims, measured over 20 seeds: the
polluted world (ii-8-pollution: pollution from t = 50, diffusion from t = 100)
against the same preset and seeds with its schedule removed, so pollution
never starts ("clean"), at tick 500 (see studio/measure.py)."""

import statistics

import measure as m
from measure import median

TICKS = 500
ONSET = 50  # pollution starts at t = 50; tick 49 is the last clean frame
HILL, PLAIN = 3, 1  # a hill site: capacity ≥ 3; the plains: ≤ 1


def _count(rows, test):
    return sum(bool(test(r)) for r in rows.values())


def verdicts(rows):
    """Each captioned claim, whether the 20 seeds support it, and why."""
    n = len(rows)
    most = 0.9 * n
    dirtier = _count(rows, lambda r: r["pol_hill"] > 1.2 * r["pol_plain"])
    fled = _count(rows, lambda r: r["hill_share"] < r["hill_share_clean"] - 0.15)
    followed = _count(rows, lambda r: r["pol_under"] > r["pol_mean"])
    ratio = statistics.median(r["pop"] / r["pop_clean"] for r in rows.values())
    fewer = _count(rows, lambda r: r["pop"] < r["pop_clean"])
    hungry = _count(rows, lambda r: r["met_hi_alive"] < r["met_hi_alive_clean"] - 0.3)
    sighted = _count(rows, lambda r: r["vis_hi_alive"] < r["vis_hi_alive_clean"] - 0.1)
    hungry_gap = median(rows, "met_hi_alive_clean") - median(rows, "met_hi_alive")
    sighted_gap = median(rows, "vis_hi_alive_clean") - median(rows, "vis_hi_alive")
    level = _count(rows, lambda r: r["gini"] < r["gini_clean"])
    return [
        ("the best land gets the dirtiest", dirtier >= most,
         f"hill sites {median(rows, 'pol_hill'):.0f} vs the plains {median(rows, 'pol_plain'):.0f}; "
         f"hills over 1.2× dirtier in {dirtier} of {n} seeds"),
        ("Flumps leave the hills", fled >= most,
         f"on the hills: {median(rows, 'hill_share49'):.0%} before the onset, {median(rows, 'hill_share'):.0%} at tick "
         f"{TICKS} ({median(rows, 'hill_share_clean'):.0%} without pollution); 15 points below it in {fled} of {n} seeds"),
        ("the mess goes wherever they go", followed >= most,
         f"the ground under Flumps {median(rows, 'pol_under'):.0f} vs {median(rows, 'pol_mean'):.0f} on average; "
         f"dirtier in {followed} of {n} seeds"),
        ("pollution costs about a quarter of the Flumps", 0.65 <= ratio <= 0.85 and fewer >= most,
         f"{median(rows, 'pop'):g} vs {median(rows, 'pop_clean'):g} Flumps (ratio {ratio:.2f}); fewer in {fewer} of {n} seeds"),
        ("the hungry go first, then the far-sighted", hungry >= most and sighted >= most and hungry_gap > sighted_gap,
         f"surviving from the onset: metabolism ≥ 3 {median(rows, 'met_hi_alive'):.0%} vs {median(rows, 'met_hi_alive_clean'):.0%} "
         f"clean ({hungry} of {n}); vision ≥ 5 {median(rows, 'vis_hi_alive'):.0%} vs {median(rows, 'vis_hi_alive_clean'):.0%} "
         f"({sighted} of {n})"),
        ("the survivors are more equal", level >= most,
         f"Gini {median(rows, 'gini'):.3f} vs {median(rows, 'gini_clean'):.3f} clean; lower in {level} of {n} seeds"),
    ]


def _hill_share(d, frame):
    agents = d.frames[frame].agents.values()
    return sum(d.capacity[a.y * d.width + a.x] >= HILL for a in agents) / max(len(agents), 1)


def seed_row(d, clean):
    """One seed's measures from its polluted dump `d` and clean dump."""
    cap, w = d.capacity, d.width
    end = d.frames[-1]
    pol = end.pollution
    row = {
        "pop": len(end.agents), "pop_clean": len(clean.frames[-1].agents),
        "pol_hill": statistics.mean(p for i, p in enumerate(pol) if cap[i] >= HILL),
        "pol_plain": statistics.mean(p for i, p in enumerate(pol) if cap[i] <= PLAIN),
        "pol_mean": statistics.mean(pol),
        "pol_under": statistics.mean(pol[a.y * w + a.x] for a in end.agents.values()),
        "pol300": statistics.mean(d.frames[300].pollution),
        "hill_share49": _hill_share(d, ONSET - 1),
        "hill_share": _hill_share(d, d.ticks), "hill_share_clean": _hill_share(clean, clean.ticks),
        "vision": statistics.mean(a.vision for a in end.agents.values()),
        "vision_clean": statistics.mean(a.vision for a in clean.frames[-1].agents.values()),
        "gini": d.stats["gini"][-1], "gini_clean": clean.stats["gini"][-1],
        "wealth": statistics.mean(a.sugar for a in end.agents.values()),
        "wealth_clean": statistics.mean(a.sugar for a in clean.frames[-1].agents.values()),
    }
    for tag, dd in (("", d), ("_clean", clean)):
        # Survival from the onset: of the Flumps alive at t = 50, how many reach the end.
        at = dd.frames[ONSET].agents
        for key, pred in (("met_hi_alive", lambda a: a.metabolism >= 3), ("vis_hi_alive", lambda a: a.vision >= 5),
                          ("met_lo_alive", lambda a: a.metabolism == 1), ("vis_lo_alive", lambda a: a.vision <= 2)):
            ids = [i for i, a in at.items() if pred(a)]
            row[key + tag] = sum(i in dd.frames[-1].agents for i in ids) / len(ids) if ids else float("nan")
    return row


def measure(tmp):
    rows = {}
    for seed in m.SEEDS:
        spec = {"preset": "ii-8-pollution", "ticks": TICKS, "seed": seed}
        d = m.shot(spec, tmp, f"polluted-{seed}")
        clean = m.shot(dict(spec, set={"schedule": []}), tmp, f"clean-{seed}")
        rows[seed] = seed_row(d, clean)
    keys = ["pop", "pop_clean", "pol_hill", "pol_plain", "pol_mean", "pol_under", "pol300",
            "hill_share49", "hill_share", "hill_share_clean",
            "met_hi_alive", "met_hi_alive_clean", "met_lo_alive", "met_lo_alive_clean",
            "vis_hi_alive", "vis_hi_alive_clean", "vis_lo_alive", "vis_lo_alive_clean",
            "vision", "vision_clean", "gini", "gini_clean", "wealth", "wealth_clean"]
    lines = [f"## ii-8-pollution vs pollution never starting, {TICKS} ticks", ""]
    lines += m.table(rows, keys)
    lines += ["", f"Typical seed: {m.typical_seed(rows, ['pop', 'hill_share', 'gini'])}.",
              f"Hill sites: capacity ≥ {HILL}; the plains: ≤ {PLAIN}. Survival runs from the onset (t = {ONSET}).",
              "Nothing in the rules removes pollution: diffusion only spreads it."]
    medians = {k: median(rows, k) for k in keys}
    return lines, verdicts(rows), {"medians": medians, "seeds": len(rows)}
