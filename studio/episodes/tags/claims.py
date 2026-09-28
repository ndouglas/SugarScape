"""The Tags episode's captioned claims (Riolo, Cohen & Axelrod 2001),
measured over 20 seeds (see studio/measure.py and
docs/superpowers/specs/2026-09-28-tags-spike.md). Every run is the paper's
length, 30,000 generations; shares are means over the whole run.

- rca-published: the paper's rules, ties to the current agent (its p. 442:
  an agent "adopts the other's tag and tolerance if the other's score is
  higher than its own"; the coin-flip reading of p. 441 doesn't give its
  tables at two pairings).
- rca-strict: the same, but a Flump gives only when the difference in shade
  is strictly below its tolerance, so twins with no tolerance needn't help.
- eh-clones-only: tolerance fixed at zero (help exact twins only),
  coin-flip ties; against rca-literal, the same coin-flip ties with
  tolerance at work.

A takeover, for both halves of the "cycle" caption: the most common exact
tag changes while at least half the Flumps hold it. The paper reports the
twins itself (p. 442: new clusters share an inherited tag; relatedness rises
to 97 %); the captions credit it.
"""

import collections
import json
import statistics
import subprocess

import measure as m
from measure import median

TICKS = 30000
# The window whose frames are read for takeovers, after the first crowd
# forms; and the window whose gifts are read.
STEP_TICKS, STEP_FROM = 10000, 100
GIFT_TICKS = 3000


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
    gaps = [r["gap"] for r in rows.values()]
    steps = [x for r in rows.values() for x in r["steps"]]
    twins_given = [r["twin_gifts"] for r in rows.values()]
    strict = [r["strict"] for r in rows.values()]
    clones = [r["clones"] for r in rows.values()]
    more = _count(rows, lambda r: r["clones"] > r["literal"])
    return [
        ("no land this time: 100 Flumps, each with a shade and a tolerance", config["agents"] == 100,
         f"{config['agents']} agents, each a tag and a tolerance in [0, 1]"),
        ("a Flump meets three others, and helps any whose shade is within its tolerance of its own",
         config["pairings"] == 3 and config["donation_test"] == "at_most",
         f"{config['pairings']} pairings a generation; a gift when the tags differ by at most the tolerance"),
        ("helping costs 0.1; being helped earns 1", (config["cost"], config["benefit"]) == (0.1, 1.0),
         f"cost {config['cost']}, benefit {config['benefit']}"),
        ("then each faces a random other, and the higher score has the child",
         config["selection"] == "tournament" and config["tie_rule"] == "current",
         f"selection {config['selection']}; ties to the current agent (the paper's p. 442)"),
        ("it works: about 74% of meetings end in a gift, as the paper says",
         round(statistics.median(gifts), 2) == 0.74 and all(0.72 <= g <= 0.76 for g in gifts),
         f"gifts per meeting: {statistics.median(gifts):.1%} (median; {min(gifts):.1%}–{max(gifts):.1%}); the "
         f"paper's Table 1: 73.6%"),
        ("the paper saw it too: most Flumps share one exact shade",
         statistics.median(twins) > 0.75 and min(twins) > 0.5,
         f"Flumps holding the dominant cluster's exact tag: {statistics.median(twins):.1%} (median; "
         f"{min(twins):.1%}–{max(twins):.1%}): {median(rows, 'cluster'):.1%} in the cluster, "
         f"{median(rows, 'related'):.1%} of it the one tag"),
        ("their tolerance is tiny; they help almost no one but their twins",
         all(r["tolerance"] < 0.03 for r in rows.values()) and statistics.median(twins_given) >= 0.9
         and min(twins_given) >= 0.85,
         f"gifts to a Flump with exactly the giver's tag, generations {STEP_FROM}–{GIFT_TICKS}: "
         f"{statistics.median(twins_given):.1%} (median; {min(twins_given):.1%}–{max(twins_given):.1%}); mean "
         f"tolerance {median(rows, 'tolerance'):.3f} (median; {min(r['tolerance'] for r in rows.values()):.3f}–"
         f"{max(r['tolerance'] for r in rows.values()):.3f})"),
        ("every few hundred generations, a new crowd takes over: always right next door",
         150 <= statistics.median(gaps) <= 800 and sum(x <= 0.03 for x in steps) >= 0.95 * len(steps)
         and max(steps) < 0.1,
         f"a takeover (the most common exact tag changing while half the Flumps hold it) every "
         f"{statistics.median(gaps):.0f} generations (median over seeds; {min(gaps):.0f}–{max(gaps):.0f}), "
         f"generations {STEP_FROM}–{STEP_TICKS}; of its {len(steps)} steps, {sum(x <= 0.03 for x in steps) / len(steps):.1%} "
         f"move 0.03 of the tag range or less (median {statistics.median(steps):.4f}), none more than {max(steps):.3f}; "
         f"the engine's own count (a new cluster over 0.01 away) finds one every "
         f"{median(rows, 'per_takeover'):.0f}"),
        ("stop twins from having to help each other, and giving collapses to 1.4%",
         round(statistics.median(strict), 3) == 0.014 and max(strict) < 0.03,
         f"strict: {statistics.median(strict):.2%} (median; {min(strict):.2%}–{max(strict):.2%})"),
        ("take tolerance away entirely, and they give even more: 75%",
         round(statistics.median(clones), 2) == 0.75 and more == n,
         f"tolerance fixed at zero, coin-flip ties: {statistics.median(clones):.1%} (median; {min(clones):.1%}–"
         f"{max(clones):.1%}), above the same coin-flip ties with tolerance at work ({median(rows, 'literal'):.1%}) in "
         f"{more} of {n}"),
    ]


def _raw_shot(spec, tmp, name):
    """A shot's dump as plain JSON (the long ones are too big for
    dump.load's objects)."""
    src, out = tmp / f"{name}.json", tmp / f"{name}.frames.json"
    src.write_text(json.dumps(spec))
    subprocess.run([m.CLI, "shot", src, "--out", out], check=True, stdout=subprocess.DEVNULL)
    raw = json.loads(out.read_text())
    out.unlink()
    return raw


def _takeovers(tmp, seed):
    """Each takeover's generation and step: the most common exact tag
    changing while at least half the Flumps hold it."""
    raw = _raw_shot({"preset": "rca-published", "ticks": STEP_TICKS, "seed": seed}, tmp, f"steps-{seed}")
    held, found = None, []
    for f in raw["frames"][STEP_FROM:]:
        tag, n = collections.Counter(a[2] for a in f["agents"]).most_common(1)[0]
        if n < 50:
            continue
        if held is not None and tag != held:
            found.append((f["tick"], abs(tag - held)))
        held = tag
    return found


def _twin_gifts(tmp, seed):
    """The share of gifts given to a Flump with exactly the giver's tag."""
    raw = _raw_shot({"preset": "rca-published", "ticks": GIFT_TICKS, "seed": seed, "gifts": True}, tmp, f"gifts-{seed}")
    same = total = 0
    for f in raw["frames"][STEP_FROM:]:
        tags = [a[2] for a in f["agents"]]
        for giver, receiver in f.get("gifts", []):
            total += 1
            same += tags[giver] == tags[receiver]
    return same / total


def seed_row(tmp, seed):
    pub = _series("rca-published", seed, tmp)
    found = _takeovers(tmp, seed)
    ticks = [t for t, _ in found]
    takeovers = pub[-1]["takeovers"]
    return {
        "gifts": _mean(pub, "donation_rate"),
        "cluster": _mean(pub, "cluster_share"),
        "related": _mean(pub, "relatedness"),
        "twins": statistics.fmean(r["cluster_share"] * r["relatedness"] for r in pub if r["tick"] >= 1),
        "tolerance": _mean(pub, "mean_tolerance"),
        "takeovers": takeovers,
        "per_takeover": TICKS / max(takeovers, 1),
        "steps": [x for _, x in found],
        "gap": statistics.fmean(b - a for a, b in zip(ticks, ticks[1:])) if len(ticks) > 1 else float(STEP_TICKS),
        "twin_gifts": _twin_gifts(tmp, seed),
        "literal": _mean(_series("rca-literal", seed, tmp), "donation_rate"),
        "strict": _mean(_series("rca-strict", seed, tmp), "donation_rate"),
        "clones": _mean(_series("eh-clones-only", seed, tmp), "donation_rate"),
    }


def measure(tmp):
    rows = {seed: seed_row(tmp, seed) for seed in m.SEEDS}
    config = m.preset_config("rca-published", tmp)
    keys = ["gifts", "cluster", "related", "twins", "tolerance", "takeovers", "per_takeover", "gap", "twin_gifts",
            "strict", "literal", "clones"]
    lines = [f"## 100 Flumps, {TICKS:,} generations, means over the run", ""]
    lines += m.table(rows, keys)
    lines += ["", f"Typical seeds: published {m.typical_seed(rows, ['gifts', 'twins', 'per_takeover'])}, "
              f"strict {m.typical_seed(rows, ['strict'])}, clones {m.typical_seed(rows, ['clones'])}."]
    medians = {k: median(rows, k) for k in keys}
    return lines, verdicts(rows, config), {"medians": medians, "seeds": len(rows)}
