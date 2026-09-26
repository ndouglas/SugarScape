"""The Seasons episode's captioned claims, measured over 20 seeds: the
seasonal world (ii-7-seasons) against the same world and seeds with seasons
off ("calm"), at tick 500 (see studio/measure.py)."""

import statistics

import measure as m
import seasons
from dump import survival, tracks
from measure import median

TICKS = 500
WINDOW = (100, TICKS)  # migrants cross hemispheres at least twice in it
HILL, PLAIN = 3, 1  # born on a hill: capacity ≥ 3; on the plains: ≤ 1


def _count(rows, test):
    return sum(bool(test(r)) for r in rows.values())


def verdicts(rows):
    """Each captioned claim, whether the 20 seeds support it, and why."""
    n = len(rows)
    most = 0.9 * n
    ratio = statistics.median(r["pop"] / r["pop_calm"] for r in rows.values())
    fewer = _count(rows, lambda r: r["pop"] < r["pop_calm"])
    follow = _count(rows, lambda r: r["migrant_summer"] > 0.55)
    none_hungry = _count(rows, lambda r: r["met_high_alive"] == 0)
    hungry_calm = _count(rows, lambda r: r["met_high_alive_calm"] > 0)
    rich_gap = median(rows, "rich_alive") - median(rows, "poor_alive")
    hill_gap = median(rows, "hill_alive") - median(rows, "plain_alive")
    need_gap = median(rows, "met_low_alive") - median(rows, "met_high_alive")
    hill_wins = _count(rows, lambda r: r["hill_alive"] > r["plain_alive"])
    thriftier = _count(rows, lambda r: r["met_alive"] < r["met_alive_calm"])
    return [
        ("many Flumps follow the summer", follow >= most,
         f"migrants spend {median(rows, 'migrant_summer'):.0%} of ticks in the summer half; above 55% in {follow} of {n} seeds"),
        ("winter is hard on the hungry: none survive", none_hungry >= most and hungry_calm >= most,
         f"no Flump with metabolism ≥ 3 survives seasons in {none_hungry} of {n} seeds; without seasons some do in {hungry_calm}"),
        ("seasons cost about a third of the Flumps", 0.6 <= ratio <= 0.8 and fewer >= most,
         f"{median(rows, 'pop'):g} vs {median(rows, 'pop_calm'):g} Flumps (ratio {ratio:.2f}); fewer in {fewer} of {n} seeds"),
        ("starting rich helps least, a hill some, needing little most",
         rich_gap < hill_gap < need_gap and hill_wins >= most,
         f"survival gaps: rich vs poor {rich_gap:+.0%}, hill vs plains {hill_gap:+.0%} ({hill_wins} of {n} seeds), "
         f"metabolism 1 vs ≥ 3 {need_gap:+.0%}"),
        ("hard times decide who's in it", thriftier >= most,
         f"survivors' mean metabolism {median(rows, 'met_alive'):.2f} vs {median(rows, 'met_alive_calm'):.2f} "
         f"without seasons; lower in {thriftier} of {n} seeds"),
    ]


def seed_row(d, calm):
    """One seed's measures from its seasonal dump `d` and calm dump."""
    cap, w, h = d.capacity, d.width, d.height
    period = d.config["seasons"]["period"]
    alive = d.frames[-1].agents
    migrants = []
    for t in tracks(d).values():
        s = seasons.switches(t, *WINDOW, h)
        if s is not None and s >= 2:
            migrants.append(seasons.summer_share(t, *WINDOW, h, period))
    row = {"pop": len(alive), "pop_calm": len(calm.frames[-1].agents)}
    row["migrant_summer"] = statistics.mean(migrants) if migrants else float("nan")
    row["migrant_share"] = len(migrants) / len(alive) if alive else float("nan")
    for tag, dd in (("", d), ("_calm", calm)):
        survivors = dd.frames[-1].agents.values()
        row["met_high_alive" + tag] = survival(dd, lambda a: a.metabolism >= 3)
        row["met_alive" + tag] = statistics.mean(a.metabolism for a in survivors)
        row["vision_alive" + tag] = statistics.mean(a.vision for a in survivors)
    row["met_low_alive"] = survival(d, lambda a: a.metabolism == 1)
    row["hill_alive"] = survival(d, lambda a: cap[a.y * w + a.x] >= HILL)
    row["plain_alive"] = survival(d, lambda a: cap[a.y * w + a.x] <= PLAIN)
    row["rich_alive"] = survival(d, lambda a: a.sugar >= 20)
    row["poor_alive"] = survival(d, lambda a: a.sugar <= 10)
    return row


def measure(tmp):
    rows = {}
    for seed in m.SEEDS:
        spec = {"preset": "ii-7-seasons", "ticks": TICKS, "seed": seed}
        d = m.shot(spec, tmp, f"seasons-{seed}")
        calm = m.shot(dict(spec, set={"seasons.enabled": False}), tmp, f"calm-{seed}")
        rows[seed] = seed_row(d, calm)
    keys = ["pop", "pop_calm", "migrant_share", "migrant_summer", "met_high_alive", "met_high_alive_calm",
            "met_low_alive", "hill_alive", "plain_alive", "rich_alive", "poor_alive",
            "met_alive", "met_alive_calm", "vision_alive", "vision_alive_calm"]
    lines = [f"## ii-7-seasons vs seasons off, {TICKS} ticks", ""]
    lines += m.table(rows, keys)
    lines += ["", f"Typical seed: {m.typical_seed(rows, ['pop', 'migrant_summer', 'met_alive'])}.",
              f"Migrants: alive through ticks {WINDOW[0]}–{WINDOW[1]} and crossing hemispheres at least twice.",
              f"Born on a hill: capacity ≥ {HILL}; on the plains: ≤ {PLAIN}. Rich: ≥ 20 sugar at the start; poor: ≤ 10."]
    medians = {k: median(rows, k) for k in keys}
    return lines, verdicts(rows), {"medians": medians, "seeds": len(rows)}
