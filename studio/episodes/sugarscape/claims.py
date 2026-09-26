"""The pilot's captioned claims, measured over 20 seeds (see studio/measure.py)."""

import statistics

import animate
import measure as m
from measure import median

HILL = 3  # a site of capacity ≥ 3 is on a hill


def verdicts(ii2, ii5, hill_share, rose):
    """Each captioned claim, whether the 20 seeds support it, and why."""
    pop25, hills = median(ii2, "pop25"), median(ii2, "on_hills")
    g0, g500, skew = median(ii5, "gini0"), median(ii5, "gini500"), median(ii5, "mean_over_median")
    top, bottom = median(ii5, "top10_share"), median(ii5, "bottom50_share")
    return [
        ("many poof early (beat 7)", pop25 < 0.8 * 400, f"median population at tick 25 is {pop25:g} of 400"),
        ("survivors crowd the hills (beats 7–8)", hills > 2 * hill_share,
         f"{hills:.0%} of survivors on hill sites, which are {hill_share:.0%} of the board"),
        ("sight up, hunger down (beat 8 dials)", rose >= 18, f"in {rose} of 20 seeds"),
        ("some are rich (beat 9)", g500 > g0 + 0.1 and skew > 1.2,
         f"Gini {g0:.2f} → {g500:.2f}; mean holding {skew:.2f} × the median"),
        ("some have much more than others (question beat)", top >= 2 * 0.1 and bottom <= 0.7 * 0.5,
         f"the richest tenth hold {top:.0%} of the sugar, the poorest half {bottom:.0%}"),
    ]


def measure(tmp):
    ii2, ii5 = {}, {}
    cap = measure_capacity(tmp)
    hill_share = sum(c >= HILL for c in cap) / len(cap)
    for seed in m.SEEDS:
        s, a = m.run("ii-2-unit", seed, 300, tmp)
        on_hills = sum(cap[int(r["y"]) * 50 + int(r["x"])] >= HILL for r in a) / max(len(a), 1)
        ii2[seed] = {
            "pop0": float(s[0]["population"]),
            "pop25": float(s[25]["population"]),
            "pop300": float(s[300]["population"]),
            "on_hills": on_hills,
            "vision0": float(s[0]["mean_vision"]),
            "vision300": float(s[300]["mean_vision"]),
            "metab0": float(s[0]["mean_metabolism"]),
            "metab300": float(s[300]["mean_metabolism"]),
        }
        s, a = m.run("ii-5-wealth", seed, 500, tmp)
        sugar = sorted(float(r["sugar"]) for r in a)
        poorest, _, richest = animate.shares(sugar)
        ii5[seed] = {
            "gini0": float(s[0]["gini"]),
            "gini500": float(s[500]["gini"]),
            "mean_over_median": statistics.mean(sugar) / statistics.median(sugar),
            "top10_share": richest,
            "bottom50_share": poorest,
        }
    rose = sum(r["vision300"] > r["vision0"] and r["metab300"] < r["metab0"] for r in ii2.values())
    lines = ["## ii-2-unit, 300 ticks (the crowd beats)", ""]
    lines += m.table(ii2, ["pop0", "pop25", "pop300", "on_hills", "vision0", "vision300", "metab0", "metab300"])
    lines += [
        "",
        f"Share of all sites with capacity ≥ {HILL}: {hill_share:.3f}.",
        f"Seeds where mean vision rose and mean metabolism fell: {rose} of 20.",
        f"Typical seed: {m.typical_seed(ii2, ['pop300', 'on_hills', 'vision300', 'metab300'])}.",
        "",
        "## ii-5-wealth, 500 ticks (the rich and question beats)",
        "",
    ]
    lines += m.table(ii5, ["gini0", "gini500", "mean_over_median", "top10_share", "bottom50_share"])
    lines += ["", f"Typical seed: {m.typical_seed(ii5, ['gini500', 'top10_share'])}."]
    return lines, verdicts(ii2, ii5, hill_share, rose)


def measure_capacity(tmp):
    return m.shot({"preset": "ii-2-unit", "ticks": 0, "set": {"population": 0}}, tmp, "landscape").capacity
