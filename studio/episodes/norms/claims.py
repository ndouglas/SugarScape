"""The Norms episode's captioned claims (Axelrod 1986; Galán & Izquierdo
2005), measured over 20 seeds (see studio/measure.py and
docs/superpowers/specs/2026-09-28-norms-spike.md).

"Established" and "collapsed" are Galán & Izquierdo's regions (§5.12): mean
boldness ≤ 2/7 with vengefulness ≥ 5/7, and boldness ≥ 6/7 with
vengefulness ≤ 1/7; Axelrod gives no thresholds. The two tie readings: when
every payoff is equal, Axelrod's text gives "an average individual one
offspring" (`keep`); Galán & Izquierdo give everyone two and remove a random
half (`drift`, the engine's default, their note 4).

- ax-norms to generation 1,000; ax-metanorms to 100 (Axelrod's horizon) and,
  run on, to 1,000,000 under each tie reading; gi-mild-metanorms to 10,000
  under each.
"""

import episode
import measure as m
from measure import median

LONG, LONG_EVERY = 1_000_000, 1000
NORMS, META, MILD, MILD_EVERY = 1000, 100, 10_000, 100


def _end(spec, tmp, name):
    """(established, collapsed) at the shot's last generation."""
    d = m.shot(spec, tmp, name)
    (tmp / f"{name}.frames.json").unlink()
    return bool(d.stats["established"][-1]), bool(d.stats["collapsed"][-1])


def _run(preset, seed, ticks, every, tmp, reading="drift", name=None):
    spec = {"preset": preset, "ticks": ticks, "seed": seed, "every": every,
            "set": {"stop_at": 0, "all_equal": reading}}
    return _end(spec, tmp, name or f"{preset}-{reading}-{seed}")


def _count(rows, key):
    return sum(bool(r[key]) for r in rows.values())


def verdicts(rows, config):
    """Each captioned claim, whether the seeds support it, and why. `config`:
    ax-metanorms's."""
    n = len(rows)
    norms_col = _count(rows, "norms_collapsed")
    held = _count(rows, "meta_established")
    long_col, long_est = _count(rows, "long_collapsed"), _count(rows, "long_established")
    his_est, his_col = _count(rows, "his_established"), _count(rows, "his_collapsed")
    mild = _count(rows, "mild_collapsed") == n and _count(rows, "mild_his_collapsed") == n
    return [
        ("each Flump has a boldness, and a vengefulness", True, "eight levels each, 0/7 to 7/7"),
        ("a bold Flump cheats when it thinks nobody is looking: it gains 3, and everyone else loses 1",
         (config["temptation"], config["hurt"]) == (3.0, -1.0), f"T {config['temptation']}, H {config['hurt']}"),
        ("anyone who sees it may punish it, if vengeful enough: −9 to the cheat, −2 to the punisher",
         (config["punishment"], config["enforcement"]) == (-9.0, -2.0),
         f"P {config['punishment']}, E {config['enforcement']}"),
        ("Axelrod, 1986: on their own, punishments fade; the cheats take over, in 20 of 20 worlds", norms_col == n,
         f"the norms game at generation {NORMS:,}: collapsed in {norms_col} of {n} (Axelrod and Galán & Izquierdo "
         f"agree)"),
        ("so he added one rule: punish anyone who looks away",
         config["metanorms"] and (config["meta_punishment"], config["meta_enforcement"]) == (-9.0, -2.0),
         "metanorms on, with the same payoffs, S and vengefulness (Axelrod's note 2 and 'critical assumption')"),
        (f"the norm holds, in {held} of 20 worlds, as he found", held >= 15,
         f"metanorms at generation {META}: established in {held} of {n} (Axelrod: 'In all five runs')"),
        (f"in 2005, Galán and Izquierdo ran it a million generations; the norm collapsed, in {long_col} of 20",
         long_col >= 15,
         f"their tie reading, generation {LONG:,}: collapsed in {long_col}, established in {long_est} of {n}"),
        ("when every Flump earns the same: Axelrod, each has one child; they, each has two, and half are removed",
         config["all_equal"] == "drift", "the engine's default is their reading; `keep` is his words"),
        (f"read his way, the norm still holds after a million generations, in {his_est} of 20", his_est > n / 2,
         f"his tie reading, generation {LONG:,}: established in {his_est}, collapsed in {his_col} of {n}"),
        ("either way, make metapunishment milder, and the norm collapses", mild,
         f"meta-payoffs divided by ten, generation {MILD:,}: collapsed in {_count(rows, 'mild_collapsed')} (their "
         f"reading) and {_count(rows, 'mild_his_collapsed')} (his) of {n}"),
    ]


def seed_row(tmp, seed):
    norms = _run("ax-norms", seed, NORMS, 1, tmp)
    meta = _run("ax-metanorms", seed, META, 1, tmp)
    long = _run("ax-metanorms", seed, LONG, LONG_EVERY, tmp)
    his = _run("ax-metanorms", seed, LONG, LONG_EVERY, tmp, "keep")
    mild = _run("gi-mild-metanorms", seed, MILD, MILD_EVERY, tmp)
    mild_his = _run("gi-mild-metanorms", seed, MILD, MILD_EVERY, tmp, "keep")
    return {
        "norms_collapsed": float(norms[1]),
        "meta_established": float(meta[0]),
        "long_established": float(long[0]), "long_collapsed": float(long[1]),
        "his_established": float(his[0]), "his_collapsed": float(his[1]),
        "mild_collapsed": float(mild[1]), "mild_his_collapsed": float(mild_his[1]),
    }


def measure(tmp):
    rows = {seed: seed_row(tmp, seed) for seed in m.SEEDS}
    config = m.preset_config("ax-metanorms", tmp)
    keys = list(next(iter(rows.values())))
    lines = ["## Twenty Flumps; G&I's regions; both tie readings", ""]
    lines += m.table(rows, keys)
    both = [s for s, r in rows.items() if r["long_collapsed"] and r["his_established"]]
    lines += ["", f"Seeds whose norm collapses under G&I's reading but holds under Axelrod's at {LONG:,}: {both}."]
    medians = {k: median(rows, k) for k in keys}
    return lines, verdicts(rows, config), {"medians": medians, "seeds": len(rows)}
