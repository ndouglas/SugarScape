"""The finale's captioned claims, and its ledger's, measured over 20 seeds
(see studio/measure.py and docs/superpowers/specs/2026-09-27-finale-episode.md).

Measured here: VI-2 (no trade) and VI-3 (trade) over 1000 ticks; II-6's
block (100 ticks); III-12's two blocks (100 ticks); III-14's civil war;
iv-3-trade's trades over 1000 ticks; IV-18's foresight over 1000 ticks.
Read from the earlier episodes' measurements, whose own claims held: Tribes'
one-tribe share, War's newcomers, Contagion's residue and fizzle.
"""

import concurrent.futures
import itertools
import json
import statistics

import episode
import measure as m
import tribes
from measure import median

TICKS = 1000
CRASH = 250  # the survey's line: a population below it has crashed
START = 500  # VI-2 and VI-3 begin with 500 agents; the book's doubling is past 1000
PERIOD = (103.5, 126.5)  # the book's ~115-year cycles, ±10 %


def _count(rows, test):
    return sum(bool(test(r)) for r in rows.values())


def _cycle(pop, lags=(103, 127), floor=0.2):
    """Whether the population over ticks 300–1000 has a cycle whose period is
    in `lags`: with its slow drift removed (a 201-tick moving average), its
    autocorrelation has a local peak in that range of at least `floor`."""
    s = pop[300:]
    half = 100
    trend = [statistics.fmean(s[max(0, i - half) : i + half + 1]) for i in range(len(s))]
    d = [v - m for v, m in zip(s, trend)]
    var = sum(v * v for v in d) / len(d) or 1.0

    def r(k):
        return sum(a * b for a, b in zip(d, d[k:])) / (len(d) - k) / var

    for k in range(lags[0], lags[1] + 1):
        here = r(k)
        if here >= floor and all(here >= r(k + j) for j in (-3, -2, -1, 1, 2, 3)):
            return True
    return False


def _beyond(x, y, width=50, height=50):
    """How far a cell is from II-6's starting block (x 0–19, y 30–49) on the
    wrapping board: the larger of its distances along each axis."""
    def gap(v, lo, hi, size):
        if lo <= v <= hi:
            return 0
        return min((v - hi) % size, (lo - v) % size)
    return max(gap(x, 0, 19, width), gap(y, 30, 49, height))


# Chapter VI's unstated details, every combination: founders' ages, their
# endowments (Chapter IV's 25–50 or Chapter III's 50–100), and how rule S's
# wealth test reads two goods.
READINGS = [
    {"lifespan.founders": ages, "sex.fertile_wealth": wealth,
     "goods.0.endowment": {"min": lo, "max": hi}, "goods.1.endowment": {"min": lo, "max": hi}}
    for ages, (lo, hi), wealth in itertools.product(("newborn", "random"), ((25, 50), (50, 100)),
                                                    ("each_good", "total", "welfare", "sugar"))
]


def _reading(tmp, reading, seed):
    """One world under one reading: (VI-2 crashed, VI-3 doubled). Crashed:
    fewer than CRASH Flumps at tick 1000 (or none); doubled: past twice its
    start at any tick."""
    vi2, _ = m.run("vi-2-no-trade", seed, TICKS, tmp, reading)
    vi3, _ = m.run("vi-3-trade", seed, TICKS, tmp, reading)
    return float(vi2[-1]["population"]) < CRASH, max(float(r["population"]) for r in vi3) > 2 * START


def readings(tmp):
    """For each reading: (worlds where VI-2 crashed, where VI-3 doubled, where
    both happened: the book's contrast)."""
    jobs = [(i, seed) for i in range(len(READINGS)) for seed in m.SEEDS]
    with concurrent.futures.ThreadPoolExecutor(8) as pool:
        results = list(pool.map(lambda job: (job[0], _reading(tmp, READINGS[job[0]], job[1])), jobs))
    out = []
    for i in range(len(READINGS)):
        mine = [r for j, r in results if j == i]
        out.append((sum(c for c, _ in mine), sum(dbl for _, dbl in mine), sum(c and dbl for c, dbl in mine)))
    return out


def earlier(name):
    return json.loads((episode.episode_dir(name) / "measurements.json").read_text())["medians"]


def verdicts(rows, sweep):
    """Each captioned claim, whether the seeds support it, and why. The
    decision rules were fixed before measuring (the review's forking-paths
    point): "never" needs 0 of 20; the readings claim needs no reading with
    the book's contrast in most worlds."""
    n = len(rows)
    no_crash = _count(rows, lambda r: r["vi2_min_late"] >= CRASH)
    doubled = _count(rows, lambda r: r["vi3_max"] > 2 * START)
    cycles = _count(rows, lambda r: r["vi3_cycle"])
    near = lambda k: 720 <= median(rows, k) <= 880  # noqa: E731
    still = _count(rows, lambda r: r["ii6_travelers"] < 0.1)
    apart = _count(rows, lambda r: r["iii12_mixed"] == 0)
    most_contrast = max(c for _, _, c in sweep)
    cleared = _count(rows, lambda r: r["v2_end"] == 0)
    meet = ("two tribes that collide (III-12): they never meet" if apart == n else
            f"two tribes that collide (III-12): in {apart} of {n} worlds they never meet")
    return [
        ("here both dip, recover and settle near 800, with trade or without",
         near("vi2_late") and near("vi3_late") and abs(median(rows, "vi2_late") - median(rows, "vi3_late")) <= 40,
         f"population over ticks 800–{TICKS}: {median(rows, 'vi2_late'):.0f} without trade, "
         f"{median(rows, 'vi3_late'):.0f} with (medians); lowest after the start {median(rows, 'vi2_dip'):.0f} and "
         f"{median(rows, 'vi3_dip'):.0f}"),
        ("no crash after the first dip in 20 of 20 worlds; no doubling in 19; no 115-year cycles",
         no_crash == n and doubled == 1 and cycles == 0,
         f"VI-2 never falls below {CRASH} after tick 300 in {no_crash} of {n}; VI-3 passes {2 * START} in {doubled}; "
         f"a cycle of 103–127 ticks (detrended autocorrelation, a local peak of at least 0.2) in {cycles}"),
        ("however we fill in what the book leaves unsaid, trade never turns a crash into a boom",
         most_contrast <= n // 2,
         f"{len(sweep)} readings (founders newborn or at random ages; endowments 25–50 or 50–100; four wealth tests for "
         f"childbearing): the book's contrast — VI-2 below {CRASH} at tick {TICKS} and VI-3 past {2 * START} — in at "
         f"most {most_contrast} of {n} worlds in any reading (crashes without trade: "
         f"{', '.join(str(c) for c, _, _ in sweep)}; doublings with trade: {', '.join(str(d) for _, d, _ in sweep)})"),
        ("waves that sweep across the land (II-6): here the Flumps stay put", still == n,
         f"Flumps that ever get 10 or more cells beyond the starting block (on the wrapping board) within 100 ticks: "
         f"{median(rows, 'ii6_travelers'):.1%} (median); under 10% in {still} of {n}"),
        (meet, True,
         f"a Blue and a Red never neighbors over 100 ticks in {apart} of {n} worlds; at most "
         f"{max(r['iii12_mixed'] for r in rows.values()):.1%} of neighbor pairs mixed in any world"),
        ("ledger: the book's civil war row — a civil war that leaves almost no one", median(rows, "iii14_pop") < 20,
         f"III-14's population at tick 100: {median(rows, 'iii14_pop'):g} of 400 (median)"),
        ("ledger: about 150,000 trades in the book; about 31,000 here", 25_000 <= median(rows, "trades") <= 37_000,
         f"iv-3-trade's trades over {TICKS} ticks: {median(rows, 'trades'):,.0f} (median)"),
        ("ledger: an endemic level of disease (V-2); it clears", cleared == n,
         f"V-2 has no sick Flump at tick {TICKS} in {cleared} of {n} worlds"),
        ("ledger: foresight selected away; lower in only 12 of 20 worlds",
         _count(rows, lambda r: r["foresight_end"] < r["foresight_start"]) == 12,
         f"IV-18's mean foresight {median(rows, 'foresight_start'):.2f} at tick 0, {median(rows, 'foresight_end'):.2f} "
         f"at {TICKS} (medians); lower at {TICKS} in {_count(rows, lambda r: r['foresight_end'] < r['foresight_start'])} of {n}"),
    ]


def seed_row(tmp, seed):
    vi2, _ = m.run("vi-2-no-trade", seed, TICKS, tmp)
    vi3, _ = m.run("vi-3-trade", seed, TICKS, tmp)
    p2 = [float(r["population"]) for r in vi2]
    p3 = [float(r["population"]) for r in vi3]
    waves = m.shot({"preset": "ii-6-waves", "ticks": 100, "seed": seed}, tmp, f"ii6-{seed}")
    start = {i: (a.x, a.y) for i, a in waves.frames[0].agents.items()}
    far = set()
    for f in waves.frames:
        for i, a in f.agents.items():
            if i in start and _beyond(a.x, a.y, waves.width, waves.height) >= 10:
                far.add(i)
    (tmp / f"ii6-{seed}.frames.json").unlink()
    blocks = m.shot({"preset": "iii-12-collision", "ticks": 100, "seed": seed}, tmp, f"iii12-{seed}")
    mixed = max(1 - tribes.neighbors_alike(f, blocks.width, blocks.height) for f in blocks.frames)
    (tmp / f"iii12-{seed}.frames.json").unlink()
    civil, _ = m.run("iii-14-combat-culture", seed, 100, tmp)
    trade, _ = m.run("iv-3-trade", seed, TICKS, tmp)
    fore, _ = m.run("iv-18-foresight", seed, TICKS, tmp)
    v2, _ = m.run("v-2-endemic", seed, TICKS, tmp)
    return {
        "vi2_late": statistics.fmean(p2[800:]), "vi3_late": statistics.fmean(p3[800:]),
        "vi2_dip": min(p2[1:300]), "vi3_dip": min(p3[1:300]),
        "vi2_min_late": min(p2[300:]), "vi3_max": max(p3), "vi3_cycle": _cycle(p3),
        "ii6_travelers": len(far) / len(start), "iii12_mixed": mixed,
        "iii14_pop": float(civil[100]["population"]),
        "trades": sum(float(r["trade_volume"]) for r in trade[1 : TICKS + 1]),
        "foresight_start": float(fore[0]["mean_foresight"]), "foresight_end": float(fore[TICKS]["mean_foresight"]),
        "v2_end": float(v2[TICKS]["infected_fraction"]),
        "pop_vi2": p2, "pop_vi3": p3,
    }


def measure(tmp):
    rows = {seed: seed_row(tmp, seed) for seed in m.SEEDS}
    sweep = readings(tmp)
    keys = ["vi2_late", "vi3_late", "vi2_dip", "vi3_dip", "vi2_min_late", "vi3_max", "ii6_travelers", "iii12_mixed",
            "iii14_pop", "trades", "foresight_start", "foresight_end", "v2_end"]
    lines = ["## Chapter VI, the waves, and the ledger's own measurements", ""]
    lines += m.table(rows, keys)
    lines += ["", f"Typical seed (VI): {m.typical_seed(rows, ['vi2_late', 'vi3_late', 'vi3_max'])}.", "",
              "## Chapter VI under every reading of its unstated details", "",
              "| founders | endowment | wealth to have children | VI-2 crashes | VI-3 doubles | both |", "|---|---|---|---|---|---|"]
    for reading, (crash, double, both) in zip(READINGS, sweep):
        e = reading["goods.0.endowment"]
        lines.append(f"| {reading['lifespan.founders']} | {e['min']}–{e['max']} | {reading['sex.fertile_wealth']} | "
                     f"{crash} | {double} | {both} |")
    medians = {k: median(rows, k) for k in keys}
    # The ledger shows trades to the nearest thousand: a median, not a count.
    medians.update(trades_rounded=round(medians["trades"], -3))
    war_m, contagion_m = earlier("war"), earlier("contagion")
    medians.update(newcomers=war_m["young"], fizzle=contagion_m["reach"],
                   readings=len(sweep), iii12_apart=_count(rows, lambda r: r["iii12_mixed"] == 0))
    lines += ["", "From the earlier episodes' measurements: "
              f"{medians['newcomers']:.0%} of III-11's dead are newcomers (War); {medians['fizzle']:.1%} catch "
              "McNeill's novel disease from another Flump (Contagion)."]
    return lines, verdicts(rows, sweep), {"medians": medians, "seeds": len(rows)}
