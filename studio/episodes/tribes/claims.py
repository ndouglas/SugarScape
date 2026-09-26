"""The Tribes episode's captioned claims, measured over 20 seeds:
iii-6-culture (rule K, the book's two tribes) at tick 3000, against the same
seeds with culture off (see studio/measure.py)."""

import statistics

import measure as m
import tribes
from measure import median

TICKS = 3000
W = H = 50


def _count(rows, test):
    return sum(bool(test(r)) for r in rows.values())


def _hill(a):
    """The two sugar hills' quarters: the north-east and the south-west."""
    if a.x >= W // 2 and a.y < H // 2:
        return "NE"
    if a.x < W // 2 and a.y >= H // 2:
        return "SW"
    return None


def verdicts(rows):
    """Each captioned claim, whether the 20 seeds support it, and why."""
    n = len(rows)
    most = 0.9 * n
    converged = _count(rows, lambda r: r["homog3000"] >= 0.9 and r["homog3000"] > r["homog0"] + 0.3)
    hills = _count(rows, lambda r: r["ne_share"] >= 0.9 and r["sw_share"] >= 0.9)
    one = _count(rows, lambda r: r["one_tribe"])
    split = _count(rows, lambda r: r["hills_differ"])
    close = _count(rows, lambda r: abs(r["pop"] - r["pop_calm"]) <= 0.05 * r["pop_calm"])
    above = _count(rows, lambda r: r["pop"] > r["pop_calm"])
    below = _count(rows, lambda r: r["pop"] < r["pop_calm"])
    return [
        # 17 of 20 seeds reach 90% by tick 3000, so the caption says "most
        # worlds", not every one.
        ("in most worlds, every neighbourhood ends up one color", converged >= 0.75 * n,
         f"neighbours in the same tribe: {median(rows, 'homog0'):.0%} at the start, {median(rows, 'homog3000'):.0%} at "
         f"tick {TICKS}; at least 90% in {converged} of {n} seeds"),
        ("each hill becomes one tribe", hills >= most, f"both hills at least 90% one tribe in {hills} of {n} seeds"),
        ("one tribe wins about half the time; otherwise the hills split",
         0.3 * n <= one <= 0.7 * n and 0.3 * n <= split <= 0.7 * n,
         f"one tribe holds at least 90% of all Flumps in {one} of {n} seeds; the hills end up different tribes in {split}"),
        ("being Red or Blue changes nothing else: just as many Flumps live either way", close >= most and max(above, below) <= 0.75 * n,
         f"population with culture {median(rows, 'pop'):g} vs {median(rows, 'pop_calm'):g} without; within 5% in {close} "
         f"of {n} seeds, higher in {above}, lower in {below}"),
    ]


def seed_row(d, calm):
    end = d.frames[-1]
    _, share = tribes.majority(end)
    ne, ne_share = tribes.majority(end, lambda a: _hill(a) == "NE")
    sw, sw_share = tribes.majority(end, lambda a: _hill(a) == "SW")
    start_lead, _ = tribes.majority(d.frames[0])
    lead, _ = tribes.majority(end)
    return {
        "homog0": tribes.neighbours_alike(d.frames[0], W, H),
        "homog1000": tribes.neighbours_alike(d.frames[1000], W, H),
        "homog3000": tribes.neighbours_alike(end, W, H),
        "global_share": share,
        "one_tribe": share >= 0.9,
        "hills_differ": ne is not None and sw is not None and ne != sw,
        "ne_share": ne_share if ne_share is not None else float("nan"),
        "sw_share": sw_share if sw_share is not None else float("nan"),
        "start_lead_wins": lead == start_lead,
        "pop": len(end.agents),
        "pop_calm": len(calm.frames[-1].agents),
    }


def measure(tmp):
    rows = {}
    for seed in m.SEEDS:
        spec = {"preset": "iii-6-culture", "ticks": TICKS, "seed": seed}
        d = m.shot(spec, tmp, f"culture-{seed}")
        calm = m.shot(dict(spec, set={"culture.enabled": False}), tmp, f"calm-{seed}")
        rows[seed] = seed_row(d, calm)
    keys = ["homog0", "homog1000", "homog3000", "global_share", "ne_share", "sw_share", "pop", "pop_calm"]
    lines = [f"## iii-6-culture vs culture off, {TICKS} ticks", ""]
    lines += m.table(rows, keys)
    one = [s for s, r in rows.items() if r["global_share"] >= 0.97]
    split = [s for s, r in rows.items() if r["hills_differ"] and r["ne_share"] >= 0.95 and r["sw_share"] >= 0.95]
    lines += ["", f"Typical seed: {m.typical_seed(rows, ['homog3000', 'global_share'])}.",
              f"Seeds where one tribe holds ≥ 97% at the end: {one}.",
              f"Seeds where the hills end as different tribes, each ≥ 95% one tribe: {split}.",
              f"The starting majority (about 51%) ends ahead in {sum(r['start_lead_wins'] for r in rows.values())} of {len(rows)}.",
              "Hills: the north-east and south-west quarters of the board. Neighbours: von Neumann pairs."]
    medians = {k: median(rows, k) for k in keys}
    medians.update(one_tribe_seeds=sum(r["one_tribe"] for r in rows.values()) / len(rows),
                   split_seeds=sum(r["hills_differ"] for r in rows.values()) / len(rows),
                   hills_one_tribe=sum(r["ne_share"] >= 0.9 and r["sw_share"] >= 0.9 for r in rows.values()) / len(rows))
    return lines, verdicts(rows), {"medians": medians, "seeds": len(rows)}
