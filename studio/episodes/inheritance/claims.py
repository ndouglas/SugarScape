"""The Inheritance episode's captioned claims, measured over 20 seeds: the
society with inheritance (iii-4-inheritance) against the same society and
seeds without it (iii-2-sex, identical but for rule I), at tick 1000
(see studio/measure.py)."""

import statistics

import lineage
import measure as m
from measure import median

TICKS = 1000
AGE = 30  # a child's fortune is read at this age
SINCE = 200  # children born from this tick on, once the founders' era has passed


def _count(rows, test):
    return sum(bool(test(r)) for r in rows.values())


def verdicts(rows):
    """Each captioned claim, whether the 20 seeds support it, and why."""
    n = len(rows)
    most = 0.9 * n
    ratio = statistics.median(r["pop"] / r["pop_plain"] for r in rows.values())
    bigger = _count(rows, lambda r: r["pop"] >= 3 * r["pop_plain"])
    unequal = _count(rows, lambda r: r["gini"] > r["gini_plain"] + 0.1 and r["top10"] > r["top10_plain"] + 0.08)
    persist = _count(rows, lambda r: r["persist"] > r["persist_plain"])
    stay = _count(rows, lambda r: r["rich_stay"] > r["rich_stay_plain"])
    gens = median(rows, "max_gen")
    return [
        ("a thousand ticks, about sixty generations", 50 <= gens <= 70, f"the deepest line reaches generation {gens:g}"),
        ("kept in the family, sugar feeds nearly four times as many", 3 <= ratio <= 4.5 and bigger >= most,
         f"{median(rows, 'pop'):.0f} vs {median(rows, 'pop_plain'):.0f} Flumps (ratio {ratio:.1f}); at least 3× in {bigger} of {n} seeds"),
        ("it is shared far less evenly", unequal >= most,
         f"Gini {median(rows, 'gini'):.2f} vs {median(rows, 'gini_plain'):.2f}; the richest tenth hold "
         f"{median(rows, 'top10'):.0%} vs {median(rows, 'top10_plain'):.0%}; in {unequal} of {n} seeds"),
        ("fortunes last", persist >= most and stay >= most,
         f"parents' wealth vs their child's at {AGE}: rank correlation {median(rows, 'persist'):.2f} vs "
         f"{median(rows, 'persist_plain'):.2f} ({persist} of {n}); children of the richest quarter end in it "
         f"{median(rows, 'rich_stay'):.0%} vs {median(rows, 'rich_stay_plain'):.0%} ({stay} of {n}); of the poorest "
         f"quarter {median(rows, 'poor_rise'):.0%} vs {median(rows, 'poor_rise_plain'):.0%}"),
    ]


def seed_row(d):
    """One run's measures: population, inequality, generations, and how a
    child's wealth at AGE follows its parents' at its birth."""
    parents = {i: ps for f in d.frames for i, (_, ps) in f.births.items()}
    born = lineage.born_at(d)
    pw, cw = [], []
    for i, ps in parents.items():
        t = born[i]
        if not ps or t < SINCE or t + AGE > d.ticks:
            continue
        before, later = d.frames[t - 1].agents, d.frames[t + AGE].agents
        if i in later and all(p in before for p in ps):
            pw.append(sum(before[p].sugar for p in ps))
            cw.append(later[i].sugar)
    top_p, low_p = sorted(pw)[3 * len(pw) // 4], sorted(pw)[len(pw) // 4]
    top_c = sorted(cw)[3 * len(cw) // 4]
    rich = [c >= top_c for p, c in zip(pw, cw) if p >= top_p]
    poor = [c >= top_c for p, c in zip(pw, cw) if p < low_p]
    end = sorted(a.sugar for a in d.frames[-1].agents.values())
    return {
        "pop": statistics.mean(len(f.agents) for f in d.frames[TICKS // 2 :]),
        "gini": statistics.mean(d.stats["gini"][TICKS // 2 :]),
        "top10": sum(end[int(len(end) * 0.9) :]) / sum(end),
        "persist": lineage.spearman(pw, cw),
        "rich_stay": sum(rich) / len(rich),
        "poor_rise": sum(poor) / len(poor),
        "max_gen": max(lineage.generations(d).values()),
        "vision": statistics.mean(a.vision for a in d.frames[-1].agents.values()),
        "metabolism": statistics.mean(a.metabolism for a in d.frames[-1].agents.values()),
        "children": len(pw),
    }


def measure(tmp):
    rows = {}
    for seed in m.SEEDS:
        heirs = seed_row(m.shot({"preset": "iii-4-inheritance", "ticks": TICKS, "seed": seed}, tmp, f"heirs-{seed}"))
        plain = seed_row(m.shot({"preset": "iii-2-sex", "ticks": TICKS, "seed": seed}, tmp, f"plain-{seed}"))
        rows[seed] = {**heirs, **{f"{k}_plain": v for k, v in plain.items()}}
    keys = ["pop", "pop_plain", "gini", "gini_plain", "top10", "top10_plain", "persist", "persist_plain",
            "rich_stay", "rich_stay_plain", "poor_rise", "poor_rise_plain", "max_gen", "max_gen_plain",
            "vision", "vision_plain", "metabolism", "metabolism_plain", "children", "children_plain"]
    lines = [f"## iii-4-inheritance vs iii-2-sex (no inheritance), {TICKS} ticks", ""]
    lines += m.table(rows, keys)
    lines += ["", f"Typical seed: {m.typical_seed(rows, ['pop', 'gini', 'persist'])}.",
              f"Population and Gini: means over ticks {TICKS // 2}–{TICKS}. Fortunes: children born from tick {SINCE} "
              f"whose parents were alive the tick before, read at age {AGE}; quarters by parents' combined wealth."]
    return lines, verdicts(rows), {"medians": {k: median(rows, k) for k in keys}, "seeds": len(rows)}
