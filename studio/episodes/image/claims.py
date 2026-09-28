"""The Reputation episode's captioned claims (Nowak & Sigmund 1998, image
scoring), measured over 20 seeds (see studio/measure.py and
docs/superpowers/specs/2026-09-28-image-scoring-spike.md).

- ns-fig-1: thresholds k from −5 to +6 drawn at random, no mutation, to
  generation 1,000 (every run has fixed by then).
- ns-fig-2: the same with mutation and 300 meetings a generation, 20,000
  generations.
- ns-fig-3-n20, -n50 and -n100: each meeting seen by ten others, each
  keeping its own tally; generations 1,001–20,000.
"""

import collections
import json
import statistics

import episode
import measure as m
import street
from measure import median

FIG1, FIG2, FIG3 = 1000, 20000, 20000
PAPER_FIG3 = {"n20": 0.90, "n50": 0.47, "n100": 0.18}


def _series(preset, seed, ticks, tmp):
    rows, agents = m.run(preset, seed, ticks, tmp)
    return [{k: float(v) if v not in ("", "NaN") else float("nan") for k, v in r.items()} for r in rows], agents


def _count(rows, test):
    return sum(bool(test(r)) for r in rows.values())


def cycles(series):
    """Collapses (cooperative strategies falling from 90 % or more to 10 %
    or less) and recoveries (the reverse)."""
    up, collapses, recoveries = None, 0, 0
    for r in series[1:]:
        x = r["cooperative"]
        if x >= 0.9:
            recoveries += up is False
            up = True
        elif x <= 0.1:
            collapses += up is True
            up = False
    return collapses, recoveries


def verdicts(rows, config, replayed=True):
    """Each captioned claim, whether the seeds support it, and why. `config`:
    ns-fig-1's; `replayed`: whether the close-up shot's meetings replay to
    its scores."""
    n = len(rows)
    k0 = _count(rows, lambda r: r["fixed"] == "k=0")
    helpful = _count(rows, lambda r: r["fixed_k"] <= 0)
    everyone = _count(rows, lambda r: r["help_end"] == 1)
    none = _count(rows, lambda r: r["help_end"] == 0)
    settled = _count(rows, lambda r: r["distinct"] == 1)
    cycling = _count(rows, lambda r: r["collapses"] >= 2 and r["recoveries"] >= 1)
    fig3 = [median(rows, key) for key in ("n20", "n50", "n100")]
    return [
        ("every Flump carries a reputation: a score from −5 to 5", config["clamp"] == 5,
         f"scores within ±{config['clamp']}, starting each generation at 0"),
        ("helping raises your score; refusing lowers it", replayed,
         "in the close-up shot, every generation's meetings, replayed with +1 for a gift and −1 for a refusal from "
         "0, give every Flump's score"),
        ("each Flump has a rule: help anyone whose score is at least k",
         config["strategies"] == ["k"] and config["information"] == "perfect",
         f"strategies {config['strategies']}, k from −5 to +6, information {config['information']}"),
        ("helping costs 0.1 and gives 1; the more a Flump earns, the more children it has",
         (config["c"], config["b"]) == (0.1, 1.0), f"c {config['c']}, b {config['b']}; offspring in proportion to payoff"),
        # NS98 show one run and give no frequency; they write that "if the
        # value is 1 or more then defection has won", more likely with fewer
        # meetings. So the captions report how often each happens here,
        # using their own split (k ≤ 0 cooperation, k ≥ 1 defection).
        ("the paper shows one run, where everyone ends up helping those who help", k0 >= 1,
         f"the discriminators (k = 0) take over in some worlds here too: {k0} of {n}"),
        ("here, everyone ends up helping in 7 of 20 worlds; in 13, nobody helps anyone",
         settled == n and helpful == everyone == 7 and none == 13,
         f"by generation {FIG1} one threshold has taken over in {settled} of {n}: k ≤ 0 (the paper's cooperation) in "
         f"{helpful}, where every meeting ends in a gift in {everyone} (k = 0 in {k0}); in {none}, no meeting does"
         f"no meeting ends in a gift"),
        ("let rules mutate, and helping rises and collapses, again and again", cycling == n,
         f"over {FIG2:,} generations, cooperative strategies (k ≤ 0) collapse at least twice and recover at least once in {cycling} of "
         f"{n} (collapses {min(r['collapses'] for r in rows.values())}–{max(r['collapses'] for r in rows.values())})"),
        ("it works best in small groups: the bigger the group, the less anyone helps",
         fig3[0] > fig3[1] > fig3[2],
         f"cooperative strategies over generations 1,001–{FIG3:,}: {fig3[0]:.0%}, {fig3[1]:.0%} and {fig3[2]:.0%} at "
         f"n = 20, 50 and 100 (medians; the paper: 90%, 47% and 18%)"),
    ]


def seed_row(tmp, seed):
    fig1, agents = _series("ns-fig-1", seed, FIG1, tmp)
    fixed, _ = collections.Counter(a["strategy"] for a in agents).most_common(1)[0]
    fig2, _ = _series("ns-fig-2", seed, FIG2, tmp)
    collapses, recoveries = cycles(fig2)
    row = {
        "fixed": fixed,
        "distinct": len({a["strategy"] for a in agents}),
        "fixed_k": int(fixed.split("=")[1]),
        "help_end": fig1[-1]["help_rate"],
        "collapses": collapses,
        "recoveries": recoveries,
        "fig2_cooperative": statistics.fmean(r["cooperative"] for r in fig2[1:]),
    }
    for key in ("n20", "n50", "n100"):
        s, _ = _series(f"ns-fig-3-{key}", seed, FIG3, tmp)
        row[key] = statistics.fmean(r["cooperative"] for r in s if r["tick"] > 1000)
    return row


def _replayed(tmp):
    spec = json.loads((episode.episode_dir("image") / "shots" / "early.json").read_text())
    d = m.shot(spec, tmp, "early")
    return all(street.replay(f.meetings, len(f.meetings), len(f.agents)) == [a[2] for a in f.agents]
               for f in d.frames[1:])


def measure(tmp):
    rows = {seed: seed_row(tmp, seed) for seed in m.SEEDS}
    config = m.preset_config("ns-fig-1", tmp)
    keys = ["fixed_k", "help_end", "collapses", "recoveries", "fig2_cooperative", "n20", "n50", "n100"]
    lines = ["## Image scoring, 100 Flumps (Fig. 3: groups of 20, 50 and 100)", ""]
    lines += m.table(rows, keys)
    lines += ["", "Fig. 1's winners by seed: " + ", ".join(f"{s} {r['fixed']}" for s, r in rows.items()) + ".",
              f"Typical seeds: fig. 2 {m.typical_seed(rows, ['collapses', 'fig2_cooperative'])}."]
    medians = {k: median(rows, k) for k in keys}
    medians.update({f"paper_{k}": v for k, v in PAPER_FIG3.items()})
    return lines, verdicts(rows, config, _replayed(tmp)), {"medians": medians, "seeds": len(rows)}
