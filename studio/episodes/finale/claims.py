"""The finale's captioned claims, and its ledger's, measured over 20 seeds
(see studio/measure.py and docs/superpowers/specs/2026-09-27-finale-episode.md).

Measured here: VI-2 (no trade) and VI-3 (trade) over 1000 ticks; II-6's
block (100 ticks); III-12's two blocks (100 ticks); III-14's civil war;
iv-3-trade's trades over 1000 ticks; IV-18's foresight over 1000 ticks.
Read from the earlier episodes' measurements, whose own claims held: Tribes'
one-tribe share, War's newcomers, Contagion's residue and fizzle.
"""

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


def _period(pop):
    """The lag (50–400 ticks) at which the population's deviations over
    ticks 300–1000 best match themselves: its cycle, if it has one."""
    s = pop[300:]
    mean = statistics.fmean(s)
    d = [v - mean for v in s]
    best, lag = None, None
    for k in range(50, 401):
        c = sum(a * b for a, b in zip(d, d[k:])) / max(len(d) - k, 1)
        if best is None or c > best:
            best, lag = c, k
    return lag


def earlier(name):
    return json.loads((episode.episode_dir(name) / "measurements.json").read_text())["medians"]


def verdicts(rows):
    """Each captioned claim, whether the 20 seeds support it, and why."""
    n = len(rows)
    no_crash = _count(rows, lambda r: r["vi2_min_late"] >= CRASH)
    doubled = _count(rows, lambda r: r["vi3_max"] > 2 * START)
    cycles = _count(rows, lambda r: PERIOD[0] <= r["vi3_period"] <= PERIOD[1])
    near = lambda k: 720 <= median(rows, k) <= 880  # noqa: E731
    still = _count(rows, lambda r: r["ii6_travelers"] < 0.1)
    # The best lag at the search's floor means no repeating rhythm at all.
    flat = _count(rows, lambda r: r["vi3_period"] == 50)
    fell = _count(rows, lambda r: r["foresight_end"] < r["foresight_start"])
    apart = _count(rows, lambda r: r["iii12_mixed"] < 0.05)
    return [
        ("here both dip, recover and settle near 800, with trade or without",
         near("vi2_late") and near("vi3_late") and abs(median(rows, "vi2_late") - median(rows, "vi3_late")) <= 40,
         f"population over ticks 800–{TICKS}: {median(rows, 'vi2_late'):.0f} without trade, "
         f"{median(rows, 'vi3_late'):.0f} with (medians); lowest after the start {median(rows, 'vi2_dip'):.0f} and "
         f"{median(rows, 'vi3_dip'):.0f}"),
        ("no crash in 20 of 20 worlds; no doubling in 19; no 115-year cycles",
         no_crash == n and doubled == 1 and cycles == 0,
         f"VI-2 never falls below {CRASH} after tick 300 in {no_crash} of {n}; VI-3 passes {2 * START} in {doubled}; "
         f"VI-3's population has no repeating rhythm in {flat} of {n}, and a cycle near 115 ticks in {cycles}"),
        ("waves that sweep across the land (II-6): here the Flumps stay put", still == n,
         f"Flumps that ever get 10 or more cells beyond the starting block within 100 ticks: "
         f"{median(rows, 'ii6_travelers'):.1%} (median); under 10% in {still} of {n}"),
        ("two tribes that collide (III-12): they never meet", apart == n,
         f"the most Blue–Red neighbor pairs in 100 ticks: {median(rows, 'iii12_mixed'):.1%} of pairs (median); under 5% "
         f"in {apart} of {n}"),
        ("ledger: the book's civil war row — a civil war that leaves almost no one", median(rows, "iii14_pop") < 20,
         f"III-14's population at tick 100: {median(rows, 'iii14_pop'):g} of 400 (median)"),
        ("ledger: about 150,000 trades in the book; about 31,000 here", 25_000 <= median(rows, "trades") <= 37_000,
         f"iv-3-trade's trades over {TICKS} ticks: {median(rows, 'trades'):,.0f} (median)"),
        ("ledger: foresight selected away; lower in only 12 of 20 worlds", fell == 12,
         f"IV-18's mean foresight {median(rows, 'foresight_start'):.2f} at tick 0, {median(rows, 'foresight_end'):.2f} "
         f"at {TICKS} (medians); lower at {TICKS} in {fell} of {n}"),
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
            if i in start and (a.x >= 30 or a.y < 20):
                far.add(i)
    (tmp / f"ii6-{seed}.frames.json").unlink()
    blocks = m.shot({"preset": "iii-12-collision", "ticks": 100, "seed": seed}, tmp, f"iii12-{seed}")
    mixed = max(1 - tribes.neighbors_alike(f, blocks.width, blocks.height) for f in blocks.frames)
    (tmp / f"iii12-{seed}.frames.json").unlink()
    civil, _ = m.run("iii-14-combat-culture", seed, 100, tmp)
    trade, _ = m.run("iv-3-trade", seed, TICKS, tmp)
    fore, _ = m.run("iv-18-foresight", seed, TICKS, tmp)
    return {
        "vi2_late": statistics.fmean(p2[800:]), "vi3_late": statistics.fmean(p3[800:]),
        "vi2_dip": min(p2[1:300]), "vi3_dip": min(p3[1:300]),
        "vi2_min_late": min(p2[300:]), "vi3_max": max(p3), "vi3_period": _period(p3),
        "ii6_travelers": len(far) / len(start), "iii12_mixed": mixed,
        "iii14_pop": float(civil[100]["population"]),
        "trades": sum(float(r["trade_volume"]) for r in trade[1 : TICKS + 1]),
        "foresight_start": float(fore[0]["mean_foresight"]), "foresight_end": float(fore[TICKS]["mean_foresight"]),
        "pop_vi2": p2, "pop_vi3": p3,
    }


def measure(tmp):
    rows = {seed: seed_row(tmp, seed) for seed in m.SEEDS}
    keys = ["vi2_late", "vi3_late", "vi2_dip", "vi3_dip", "vi2_min_late", "vi3_max", "vi3_period", "ii6_travelers",
            "iii12_mixed", "iii14_pop", "trades", "foresight_start", "foresight_end"]
    lines = ["## Chapter VI, the waves, and the ledger's own measurements", ""]
    lines += m.table(rows, keys)
    lines += ["", f"Typical seed (VI): {m.typical_seed(rows, ['vi2_late', 'vi3_late', 'vi3_max'])}."]
    medians = {k: median(rows, k) for k in keys}
    tribes_m, war_m, contagion_m = earlier("tribes"), earlier("war"), earlier("contagion")
    # The ledger shows trades to the nearest thousand: a median, not a count.
    medians.update(trades_rounded=round(medians["trades"], -3))
    medians.update(one_tribe_share=tribes_m["one_tribe_seeds"], newcomers=war_m["young"],
                   residue=contagion_m["v1_end"], fizzle=contagion_m["reach"])
    lines += ["", "From the earlier episodes' measurements: "
              f"one tribe takes the world in {medians['one_tribe_share']:.0%} of worlds (Tribes); "
              f"{medians['newcomers']:.0%} of III-11's dead are newcomers (War); "
              f"{medians['residue']:.1%} still sick at t = 1000 in V-1, and {medians['fizzle']:.1%} catch McNeill's "
              "novel disease (Contagion)."]
    return lines, verdicts(rows), {"medians": medians, "seeds": len(rows)}
