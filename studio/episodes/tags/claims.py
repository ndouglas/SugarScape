"""The Tags episode's captioned claims (Riolo, Cohen & Axelrod 2001),
measured over 20 seeds (see studio/measure.py and
docs/superpowers/specs/2026-09-28-tags-spike.md). Every run is the paper's
length, 30,000 generations; shares are means over the whole run.

- rca-published: the paper's rules, with ties to the current agent (the
  only reading that matches its tables).
- rca-strict: the same, but a Flump gives only when the difference in shade
  is strictly below its tolerance, so twins with no tolerance needn't help.
- eh-clones-only: tolerance fixed at zero (help exact twins only),
  coin-flip ties.
"""

import collections
import statistics

import measure as m
from measure import median

TICKS = 30000
# The window whose frames are read for takeovers' steps, after the first
# crowd forms.
STEP_TICKS, STEP_FROM = 6000, 100


def _series(preset, seed, tmp):
    rows, _ = m.run(preset, seed, TICKS, tmp)
    return [{k: float(v) if v not in ("", "NaN") else float("nan") for k, v in r.items()} for r in rows]


def _mean(rows, key):
    values = [r[key] for r in rows if r["tick"] >= 1 and r[key] == r[key]]
    return statistics.fmean(values)


def _count(rows, test):
    return sum(bool(test(r)) for r in rows.values())


def verdicts(rows, config):
    """Each captioned claim, whether the seeds support it, and why. `config`:
    rca-published's."""
    n = len(rows)
    gifts = [r["gifts"] for r in rows.values()]
    twins = [r["twins"] for r in rows.values()]
    per = [r["per_takeover"] for r in rows.values()]
    strict = [r["strict"] for r in rows.values()]
    clones = [r["clones"] for r in rows.values()]
    more = _count(rows, lambda r: r["clones"] > r["gifts"])
    return [
        ("no land this time: 100 Flumps, each with a shade and a tolerance", config["agents"] == 100,
         f"{config['agents']} agents, each a tag and a tolerance in [0, 1]"),
        ("a Flump meets three others, and helps any whose shade is within its tolerance of its own",
         config["pairings"] == 3 and config["donation_test"] == "at_most",
         f"{config['pairings']} pairings a generation; a gift when the tags differ by at most the tolerance"),
        ("helping costs 0.1; being helped earns 1", (config["cost"], config["benefit"]) == (0.1, 1.0),
         f"cost {config['cost']}, benefit {config['benefit']}"),
        ("then each faces a random other, and the winner's child takes the loser's place",
         config["selection"] == "tournament" and config["tie_rule"] == "current",
         f"selection {config['selection']}; ties to the current agent (the paper never says: the finale's)"),
        ("it works: about 74% of meetings end in a gift, as the paper says",
         round(statistics.median(gifts), 2) == 0.74 and all(0.72 <= g <= 0.76 for g in gifts),
         f"gifts per meeting: {statistics.median(gifts):.1%} (median; {min(gifts):.1%}–{max(gifts):.1%}); the "
         f"paper's Table 1: 73.6%"),
        ("but look where they stand: most Flumps share one exact shade",
         statistics.median(twins) > 0.75 and min(twins) > 0.5,
         f"Flumps holding the dominant cluster's exact tag: {statistics.median(twins):.1%} (median; "
         f"{min(twins):.1%}–{max(twins):.1%}): {median(rows, 'cluster'):.1%} in the cluster, "
         f"{median(rows, 'related'):.1%} of it the one tag"),
        ("their tolerance is tiny; they help almost no one but their twins",
         all(r["tolerance"] < 0.03 for r in rows.values()),
         f"mean tolerance {median(rows, 'tolerance'):.3f} (median; "
         f"{min(r['tolerance'] for r in rows.values()):.3f}–{max(r['tolerance'] for r in rows.values()):.3f}), "
         f"against a start spread over 0–1"),
        ("every thousand generations or so, a new shade takes over: always the one next door",
         700 <= statistics.median(per) <= 1400 and max(r["step"] for r in rows.values()) < 0.1,
         f"one takeover every {statistics.median(per):.0f} generations (median; {min(per):.0f}–{max(per):.0f}); "
         f"{median(rows, 'takeovers'):.0f} a run; each time the most common exact tag changes (holding 50+ Flumps, "
         f"generations {STEP_FROM}–{STEP_TICKS}), it moves {median(rows, 'step_median'):.3f} of the tag range "
         f"(median over seeds), never more than {max(r['step'] for r in rows.values()):.3f}"),
        ("stop twins from having to help each other, and giving collapses to 1.4%",
         round(statistics.median(strict), 3) == 0.014 and max(strict) < 0.03,
         f"strict: {statistics.median(strict):.2%} (median; {min(strict):.2%}–{max(strict):.2%})"),
        ("take tolerance away entirely, and they give even more: 75%",
         round(statistics.median(clones), 2) == 0.75 and more == n,
         f"tolerance fixed at zero: {statistics.median(clones):.1%} (median; {min(clones):.1%}–{max(clones):.1%}), "
         f"above the paper's rules in {more} of {n}"),
    ]


def _steps(tmp, seed):
    """How far the most common exact tag moves each time it changes, while
    it holds at least half the Flumps."""
    d = m.shot({"preset": "rca-published", "ticks": STEP_TICKS, "seed": seed}, tmp, f"steps-{seed}")
    held, steps = None, []
    for f in d.frames[STEP_FROM:]:
        tag, n = collections.Counter(a.tag for a in f.agents).most_common(1)[0]
        if n < 50:
            continue
        if held is not None and tag != held:
            steps.append(abs(tag - held))
        held = tag
    (tmp / f"steps-{seed}.frames.json").unlink()
    return steps


def seed_row(tmp, seed):
    pub = _series("rca-published", seed, tmp)
    steps = _steps(tmp, seed)
    takeovers = pub[-1]["takeovers"]
    return {
        "gifts": _mean(pub, "donation_rate"),
        "cluster": _mean(pub, "cluster_share"),
        "related": _mean(pub, "relatedness"),
        "twins": statistics.fmean(r["cluster_share"] * r["relatedness"] for r in pub if r["tick"] >= 1),
        "tolerance": _mean(pub, "mean_tolerance"),
        "takeovers": takeovers,
        "per_takeover": TICKS / max(takeovers, 1),
        "step": max(steps, default=0.0),
        "step_median": statistics.median(steps) if steps else 0.0,
        "strict": _mean(_series("rca-strict", seed, tmp), "donation_rate"),
        "clones": _mean(_series("eh-clones-only", seed, tmp), "donation_rate"),
    }


def measure(tmp):
    rows = {seed: seed_row(tmp, seed) for seed in m.SEEDS}
    config = m.preset_config("rca-published", tmp)
    keys = ["gifts", "cluster", "related", "twins", "tolerance", "takeovers", "per_takeover", "step", "step_median",
            "strict", "clones"]
    lines = [f"## 100 Flumps, {TICKS:,} generations, means over the run", ""]
    lines += m.table(rows, keys)
    lines += ["", f"Typical seeds: published {m.typical_seed(rows, ['gifts', 'twins', 'per_takeover'])}, "
              f"strict {m.typical_seed(rows, ['strict'])}, clones {m.typical_seed(rows, ['clones'])}."]
    medians = {k: median(rows, k) for k in keys}
    return lines, verdicts(rows, config), {"medians": medians, "seeds": len(rows)}
