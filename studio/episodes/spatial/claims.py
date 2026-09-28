"""The Spatial games episode's captioned claims, measured over 20 seeds (see
studio/measure.py and docs/superpowers/specs/2026-09-27-spatial-games-spike.md).
Every board is the one filmed: 99 × 99, fixed edges unless noted.

- The rules, from the close-up's hand-made 7 × 7 board (its scores and its
  copies, generation by generation).
- nm-3-kaleidoscope: one cheat at the center, b = 1.9, all at once.
- nm-1b-chaos on 99 × 99: 10 % or 40 % cheats scattered at random, b = 1.9;
  and at b = 1.77.
- hg-async-kaleidoscope: the kaleidoscope, one Flump at a time; and at b = 1.7.
- nbm-continuous and nbm-discrete on 99 × 99: 50 % cheats, periodic edges,
  b = 1.71, one at a time and all at once.
"""

import json
import statistics

import episode
import lattice
import measure as m
from measure import median

N = 99
BIG = {"width": N, "height": N}
LATE = (201, 300)
WHY = (100, 300)
ASYNC_TICKS = 200
LONG = 1000


def _series(preset, seed, ticks, tmp, changes=None):
    rows, _ = m.run(preset, seed, ticks, tmp, changes)
    return [{k: float(v) for k, v in r.items()} for r in rows]


def _mean(series, key, span):
    return statistics.fmean(r[key] for r in series if span[0] <= r["tick"] <= span[1])


def _count(rows, test):
    return sum(bool(test(r)) for r in rows.values())


def verdicts(rows, facts):
    """Each captioned claim, whether the seeds support it, and why. `rows`:
    seed → measures; `facts`: what holds of every seed at once (the rules,
    checked on the close-up's board, and the kaleidoscope, which has no
    randomness)."""
    n = len(rows)
    thirds = [r[k] for r in rows.values() for k in ("third_10", "third_40")]
    ahead = _count(rows, lambda r: r["ahead"] == 1)
    tempt = [r["tempt"] for r in rows.values()]
    lost = _count(rows, lambda r: r["all_d_at"] is not None)
    held = _count(rows, lambda r: r["async_17_min"] > 0.9)
    both = _count(rows, lambda r: r["below_async"] > 0.5 and r["below_sync"] > 0.5)
    frontier = median(rows, "frontier")
    return [
        ("every Flump plays each of its neighbors; two helpers earn 1 each, a cheat facing a helper almost 2, "
         "and the helper nothing", facts["scores_follow"] and facts["b"] == 1.9,
         f"every score on the close-up's board is 1 per helper beside a helper (and 1 for itself) and "
         f"{facts['b']} per helper beside a cheat, in all {facts['board_frames']} generations"),
        ("then every Flump copies whichever neighbor earned the most", facts["copies_follow"],
         f"each square's next side is its best-earning candidate's (itself or a neighbor; ties keep it), in all "
         f"{facts['board_frames'] - 1} steps of the close-up"),
        ("one cheat, in a world of 9,800 helpers", facts["start"] == (1, 9800),
         f"the kaleidoscope starts with {facts['start'][0]} cheat and {facts['start'][1]:,} helpers"),
        ("cheating spreads, but never wins, and never settles",
         facts["same_every_seed"] and facts["symmetric"] and facts["min_c"] > 0.1 and facts["still"] == 0,
         f"the same in all {n} seeds (no randomness), four-fold symmetric every generation to "
         f"{facts['symmetric_to']}, a cheat at the edge by t = {facts['edge']}; over t = 50–{LONG}, helpers never "
         f"fall below {facts['min_c']:.0%}, and {facts['still']} generations pass without a change "
         f"({facts['changed']:.0%} change sides each generation on average)"),
        ("scatter the cheats anywhere: about a third of the Flumps end up helping",
         all(0.28 <= t <= 0.38 for t in thirds),
         f"from 10 % or 40 % cheats at random, helpers over t = {LATE[0]}–{LATE[1]}: "
         f"{statistics.median(thirds):.1%} (median; {min(thirds):.1%}–{max(thirds):.1%}, all {len(thirds)} runs)"),
        # The storyboard's first sentence, "a cheat beside helpers out-earns
        # them", failed (the cheat wins about half the frontier's pairs): it
        # was dropped, and the frontier is reported here as context.
        ("on average, helpers earn more than cheats, every generation", ahead == n,
         f"the average helper out-earns the average cheat in every generation of t = {WHY[0]}–{WHY[1]} in {ahead} "
         f"of {n} (along the frontier, the cheat out-earns the helper beside it in only {frontier:.0%} of pairs: "
         f"median over every 20th generation; {min(r['frontier'] for r in rows.values()):.0%}–"
         f"{max(r['frontier'] for r in rows.values()):.0%})"),
        ("make cheating a little less tempting, and three in four help", all(0.7 <= t <= 0.8 for t in tempt),
         f"b = 1.77: helpers over t = {LATE[0]}–{LATE[1]} {statistics.median(tempt):.1%} (median; "
         f"{min(tempt):.1%}–{max(tempt):.1%})"),
        ("here, everyone moves at once; Huberman and Glance let the Flumps move one at a time",
         facts["kaleidoscope_update"] == "synchronous" and facts["clock_update"] == "asynchronous",
         f"the kaleidoscope updates {facts['kaleidoscope_update']}; the next shot, the same start, "
         f"{facts['clock_update']}"),
        ("one cheat takes the whole world, in 20 of 20", lost == n,
         f"one at a time, b = 1.9: no helper left by t = {ASYNC_TICKS} in {lost} of {n} (at t = "
         f"{min(r['all_d_at'] or ASYNC_TICKS for r in rows.values()):g}–"
         f"{max(r['all_d_at'] or 0 for r in rows.values()):g})"),
        ("unless cheating is less tempting: then helpers survive either way", both == n and held == n,
         f"b = 1.71 from 50 % cheats: helpers at t = {ASYNC_TICKS} one at a time {median(rows, 'below_async'):.0%}, all "
         f"at once {median(rows, 'below_sync'):.0%} (medians), a majority both ways in {both} of {n}; the one-at-a-time "
         f"kaleidoscope at b = 1.7 keeps over 90 % helping to t = 400 in {held} of {n}"),
    ]


def _board_facts(tmp):
    """The rules, checked on the close-up's board: scores, and copies."""
    spec = episode_shot("board")
    d = m.shot(spec, tmp, "board")
    w, h, b = d.width, d.height, d.config["b"]
    scores_ok = copies_ok = True
    for k, f in enumerate(d.frames):
        s = f.strategies
        for i, c in enumerate(s):
            x, y = i % w, i // w
            near = [ny * w + nx for dx, dy in lattice.NEIGHBORS for nx, ny in [(x + dx, y + dy)]
                    if 0 <= nx < w and 0 <= ny < h]
            helpers = sum(s[j] == "C" for j in near)
            scores_ok &= abs(f.scores[i] - (helpers + 1 if c == "C" else b * helpers)) < 1e-9
            if k < d.ticks:
                best = max(f.scores[j] for j in [i, *near])
                sides = {s[j] for j in [i, *near] if f.scores[j] == best}
                expected = c if len(sides) > 1 else sides.pop()
                copies_ok &= d.frames[k + 1].strategies[i] == expected
    return {"scores_follow": scores_ok, "copies_follow": copies_ok, "b": b, "board_frames": len(d.frames)}


def episode_shot(name):
    return json.loads((episode.episode_dir("spatial") / "shots" / f"{name}.json").read_text())


def _kaleidoscope_facts(tmp, rows):
    d = m.shot(episode_shot("kaleidoscope"), tmp, "kaleidoscope")
    first = d.frames[0].strategies
    sym = [lattice.symmetric(f.strategies, d.width, d.height) for f in d.frames]
    edge = next(f.tick for f in d.frames if lattice.reaches_edge(f.strategies, d.width, d.height))
    clock = m.shot(episode_shot("clock"), tmp, "clock-config")
    long = _series("nm-3-kaleidoscope", 1, LONG, tmp)
    late = [r for r in long if r["tick"] >= 50]
    return {
        "start": (first.count("D"), first.count("C")),
        "symmetric": all(sym),
        "symmetric_to": d.ticks,
        "edge": edge,
        "same_every_seed": len({r["kaleidoscope"] for r in rows.values()}) == 1,
        "min_c": min(r["fraction_c"] for r in late),
        "still": sum(r["changed"] == 0 for r in late),
        "changed": statistics.fmean(r["changed"] for r in late),
        "kaleidoscope_update": d.config["update"],
        "clock_update": clock.config["update"],
    }


def _frontier(tmp, seed):
    """The median share, over every 20th generation of WHY, of frontier pairs
    in which the cheat out-earns the helper (the scatter shot, with scores)."""
    spec = {"preset": "nm-1b-chaos", "ticks": WHY[1], "seed": seed, "scores": True, "set": BIG}
    d = m.shot(spec, tmp, f"frontier-{seed}")
    shares = [lattice.boundary(d.frames[k], d.width, d.height)[1] for k in range(WHY[0], WHY[1] + 1, 20)]
    (tmp / f"frontier-{seed}.frames.json").unlink()
    return statistics.median(shares)


def seed_row(tmp, seed):
    scatter = _series("nm-1b-chaos", seed, LATE[1], tmp, {**BIG})
    forty = _series("nm-1b-chaos", seed, LATE[1], tmp, {**BIG, "defectors": 0.4})
    tempt = _series("nm-1b-chaos", seed, LATE[1], tmp, {**BIG, "b": 1.77})
    clock = _series("hg-async-kaleidoscope", seed, ASYNC_TICKS, tmp)
    held = _series("hg-async-kaleidoscope", seed, 400, tmp, {"b": 1.7})
    below_async = _series("nbm-continuous", seed, ASYNC_TICKS, tmp, {**BIG})
    below_sync = _series("nbm-discrete", seed, ASYNC_TICKS, tmp, {**BIG})
    kal = _series("nm-3-kaleidoscope", seed, 260, tmp)
    why = [r for r in scatter if WHY[0] <= r["tick"] <= WHY[1]]
    return {
        "third_10": _mean(scatter, "fraction_c", LATE),
        "third_40": _mean(forty, "fraction_c", LATE),
        "ahead": float(all(r["mean_payoff_c"] > r["mean_payoff_d"] for r in why)),
        "frontier": _frontier(tmp, seed),
        "tempt": _mean(tempt, "fraction_c", LATE),
        "all_d_at": next((r["tick"] for r in clock if r["fraction_c"] == 0), None),
        "async_17_min": min(r["fraction_c"] for r in held),
        "below_async": below_async[-1]["fraction_c"],
        "below_sync": below_sync[-1]["fraction_c"],
        "kaleidoscope": tuple(r["fraction_c"] for r in kal),
    }


def measure(tmp):
    rows = {seed: seed_row(tmp, seed) for seed in m.SEEDS}
    facts = {**_board_facts(tmp), **_kaleidoscope_facts(tmp, rows)}
    keys = ["third_10", "third_40", "frontier", "tempt", "async_17_min", "below_async", "below_sync"]
    clocked = {s: {**r, "all_d_at": r["all_d_at"] or ASYNC_TICKS} for s, r in rows.items()}
    lines = ["## 99 × 99 boards (the kaleidoscope's 9,801 Flumps)", ""]
    lines += m.table(rows, keys)
    lines += m.table(clocked, ["all_d_at"])[2:]
    lines += ["", f"Typical seeds: scatter {m.typical_seed(rows, ['third_10', 'frontier'])}, "
              f"tempt {m.typical_seed(rows, ['tempt'])}, clock {m.typical_seed(clocked, ['all_d_at'])}, "
              f"below {m.typical_seed(rows, ['below_async'])}.",
              f"The kaleidoscope: {facts['start'][0]} cheat, {facts['start'][1]} helpers; symmetric to "
              f"t = {facts['symmetric_to']}: {facts['symmetric']}; a cheat at the edge by t = {facts['edge']}."]
    medians = {k: median(rows, k) for k in keys}
    return lines, verdicts(rows, facts), {"medians": medians, "seeds": len(rows)}
