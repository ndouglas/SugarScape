"""The Friends and strangers episode's captioned claims (Cohen, Riolo &
Axelrod 2001), measured over 20 seeds (see studio/measure.py and
docs/superpowers/specs/2026-09-28-social-structure-spike.md), with the
paper's own measures: the mean payoff per move over the last 1,000 of 2,500
periods, and the share of time cooperation remains high once reached (high:
a mean payoff of at least 2.3, which the paper's p. 20 implies).

- cra-rwr (strangers every period), cra-2dk (the torus), cra-frn (fixed
  random partners), and cra-ffr-01, -03 and -05 (each fixed partner swapped
  for a stranger with that chance each period).
"""

import statistics

import measure as m
from measure import median

TICKS, LAST = 2500, 1000
STRETCH = 50  # periods: a flicker needs a high and a low stretch at least this long
PAPER = {"rwr": 1.091, "2dk": 2.557, "frn": 2.480, "ffr1": 2.385, "ffr3": 2.100, "ffr5": 1.257}
PRESETS = {"rwr": "cra-rwr", "2dk": "cra-2dk", "frn": "cra-frn", "ffr1": "cra-ffr-01", "ffr3": "cra-ffr-03",
           "ffr5": "cra-ffr-05"}


def _series(preset, seed, tmp):
    rows, _ = m.run(preset, seed, TICKS, tmp)
    return [{k: float(v) if v not in ("", "NaN") else float("nan") for k, v in r.items()} for r in rows]


def _stretches(rows, high):
    """The longest run of consecutive periods (after the first 1,000) that are
    high, or not."""
    best = cur = 0
    for r in rows[1001:]:
        cur = cur + 1 if bool(r["high"]) == high else 0
        best = max(best, cur)
    return best


def verdicts(rows, config):
    """Each captioned claim, whether the seeds support it, and why. `config`:
    cra-2dk's."""
    n = len(rows)
    med = {k: median(rows, f"pay_{k}") for k in PRESETS}
    remain = {k: median(rows, f"remain_{k}") for k in PRESETS}
    reached_2dk = int(sum(r["reached_2dk"] for r in rows.values()))
    reached_frn = int(sum(r["reached_frn"] for r in rows.values()))
    flicker = int(sum(r["flicker"] for r in rows.values()))
    worst = max(abs(med[k] - PAPER[k]) for k in PRESETS)
    same_order = sorted(PRESETS, key=lambda k: med[k]) == sorted(PRESETS, key=lambda k: PAPER[k])
    return [
        ("256 Flumps, each with three habits: how often it helps first, after being helped, and after being cheated",
         config["agents"] == 256, "y, p and q, each a probability"),
        ("each period, each plays four rounds with four partners", (config["partners"], config["moves"]) == (4, 4),
         f"{config['partners']} partners, {config['moves']} moves a game"),
        ("then each copies its best-scoring partner, if it did better", config["judge_error"] == 0.1,
         f"copied when strictly better, misjudged with probability {config['judge_error']}, and each habit nudged "
         f"with probability {config['mutation']}"),
        ("meet strangers every period, and cooperation almost never takes hold", med["rwr"] < 1.2 and remain["rwr"] < 0.05,
         f"strangers: mean payoff {med['rwr']:.3f} (median; the paper 1.091), high {remain['rwr']:.1%} of the time "
         f"once reached (the paper 1.5 %)"),
        ("keep the same four neighbors, and it takes hold in every world, and stays",
         reached_2dk == n and remain["2dk"] >= 0.99,
         f"the torus: high cooperation reached in {reached_2dk} of {n}, and high {remain['2dk']:.1%} of the time "
         f"after (median; the paper 99.7 %); mean payoff {med['2dk']:.3f} (the paper 2.557)"),
        ("keep the same partners, scattered anywhere: nearly as good; no geography needed",
         reached_frn == n and med["2dk"] - med["frn"] <= 0.15,
         f"fixed random partners: {med['frn']:.3f} against the torus's {med['2dk']:.3f} (the paper 2.480 and 2.557); "
         f"reached in {reached_frn} of {n}"),
        ("swap a tenth of them each period, and it mostly holds", remain["ffr1"] >= 0.75,
         f"a tenth swapped: high {remain['ffr1']:.1%} of the time once reached (the paper 84.4 %); payoff "
         f"{med['ffr1']:.3f} (2.385)"),
        # The storyboard's "it flickers" (a high and a low stretch of 50+
        # periods after period 1,000 in 15+ of 20 runs) failed, 11 of 20;
        # the caption says what holds instead.
        ("swap 30%, and it holds only about a third of the time", 0.25 <= remain["ffr3"] <= 0.45,
         f"30 % swapped: high {remain['ffr3']:.1%} of the time once reached (the paper 40.2 %); payoff "
         f"{med['ffr3']:.3f} (2.100); a high and a low stretch of {STRETCH}+ periods after period 1,000 in "
         f"{flicker:.0f} of {n}"),
        ("half, and it collapses", med["ffr5"] < 1.5 and remain["ffr5"] < 0.15,
         f"half swapped: payoff {med['ffr5']:.3f} (the paper 1.257), high {remain['ffr5']:.1%} of the time once "
         f"reached (6.1 %)"),
        ("that's what Cohen, Riolo and Axelrod found in 2001, row by row", worst <= 0.1 and same_order,
         f"every structure's median payoff within {worst:.3f} of Table 2's, in the same order: "
         + ", ".join(f"{k} {med[k]:.3f}/{PAPER[k]}" for k in PRESETS)),
    ]


def seed_row(tmp, seed):
    row = {}
    for key, preset in PRESETS.items():
        s = _series(preset, seed, tmp)
        row[f"pay_{key}"] = statistics.fmean(r["mean_payoff"] for r in s if r["tick"] > TICKS - LAST)
        remain = s[-1]["share_high_since"]
        row[f"remain_{key}"] = remain if remain == remain else 0.0
        row[f"reached_{key}"] = float(any(r["high"] for r in s[1:]))
        if key == "ffr3":
            row["flicker"] = float(_stretches(s, True) >= STRETCH and _stretches(s, False) >= STRETCH)
    return row


def measure(tmp):
    rows = {seed: seed_row(tmp, seed) for seed in m.SEEDS}
    config = m.preset_config("cra-2dk", tmp)
    keys = [f"pay_{k}" for k in PRESETS] + [f"remain_{k}" for k in PRESETS]
    lines = ["## 256 Flumps, 2,500 periods; payoff over the last 1,000", ""]
    lines += m.table(rows, keys)
    lines += ["", "Typical seeds: " + ", ".join(f"{k} {m.typical_seed(rows, [f'pay_{k}'])}" for k in PRESETS) + "."]
    medians = {k: median(rows, k) for k in keys}
    medians.update({f"paper_{k}": v for k, v in PAPER.items()})
    return lines, verdicts(rows, config), {"medians": medians, "seeds": len(rows)}
