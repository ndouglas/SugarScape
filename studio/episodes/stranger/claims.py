"""The Cooperation finale's captioned claims, and its ledger's (see
docs/superpowers/specs/2026-09-28-cooperation-finale-spike.md).

Read from the earlier episodes' measurements, over 20 seeds each, and only
from claims that held there: a row whose source claim no longer holds (or
is missing) fails here too. Measured here: Epstein's Run 4 under the book's
rule, the working paper's, and the working paper's with founders who start
with nothing, to cycle 500.
"""

import json
import re

import episode
import measure as m

RUN4 = {"max_age": 100, "r": 1.0}
READINGS = {"book": ("dpd-run-4", None), "working": ("dpd-working-paper", RUN4), "closest": ("dpd-closest", RUN4)}
TICKS = 500
EXTINCT = 15  # of 20: "under either rule" only if every reading dies out at least this often (fixed first)


def medians(name):
    return json.loads((episode.episode_dir(name) / "measurements.json").read_text())["medians"]


def held(name, start):
    """The measured line of an earlier episode's claim that starts with
    `start`, which must have held there."""
    for line in (episode.episode_dir(name) / "measurements.md").read_text().splitlines():
        if line.startswith("- DOES NOT HOLD: " + start):
            raise ValueError(f"{name}: {start!r} no longer holds")
        if line.startswith("- holds: " + start):
            return line
    raise ValueError(f"{name}: no claim starting {start!r}")


def number(pattern, line):
    return float(re.search(pattern, line).group(1))


def sources():
    """Everything the finale shows, from the earlier episodes."""
    sp, et, tg, im, fr = (medians(n) for n in ("spatial", "ethno", "tags", "image", "friends"))
    clock = held("spatial", "one cheat takes the whole world")
    living = held("living", "Epstein counted 779 helpers")
    blind = held("ethno", "the paper: at double cost")
    here = held("image", "here, everyone ends up helping")
    meta = held("norms", "the norm holds")
    long = held("norms", "in 2005, Galán and Izquierdo")
    his = held("norms", "read his way")
    # Each figure below comes from a claim that held; `held` checks the rest.
    for name, start in (("spatial", "scatter a few cheats anywhere"), ("spatial", "Nowak, Bonhoeffer and May"),
                        ("ethno", "favoritism wins"), ("ethno", "why? their neighbors share ancestors"),
                        ("tags", "it works"), ("tags", "their tolerance is tiny"),
                        ("image", "it works best in small groups"), ("friends", "keep the same partners"),
                        ("friends", "meet strangers every period"), ("friends", "that's what Cohen, Riolo")):
        held(name, start)
    thirds = [sp[k] for k in ("third_2", "third_10", "third_25")]
    rows = ("rwr", "2dk", "frn", "ffr1", "ffr3", "ffr5")
    return {
        "third_lo": min(thirds), "third_hi": max(thirds),
        "hg_lost": number(r"in (\d+) of 20", clock),
        "nbm_async": sp["below_async"], "nbm_sync": sp["below_sync"],
        "own": et["own"], "kin_help": et["kin_help"],
        "seeing2": et["seeing2"], "seeing2_lo": number(r"seeing [\d.]+% \(median; ([\d.]+)%", blind) / 100,
        "gifts": tg["gifts"], "twin_gifts": tg["twin_gifts"],
        "n20": im["n20"], "n50": im["n50"], "n100": im["n100"],
        "everyone": number(r"every meeting ends in a gift in (\d+)", here),
        "meta": number(r"established in (\d+) of 20", meta),
        "long_est": number(r"established in (\d+) of 20", long),
        "his_est": number(r"established in (\d+),", his),
        "published": number(r"published rules: (\d+)", living),
        "working_only": number(r"rule alone: (\d+)", living),
        "working": number(r"no starting wealth: (\d+)", living),
        "frn": fr["pay_frn"], "frn_remain": fr["remain_frn"], "rwr": fr["pay_rwr"], "rwr_remain": fr["remain_rwr"],
        "cra_worst": max(abs(fr[f"pay_{k}"] - fr[f"paper_{k}"]) for k in rows),
        "cra_order": sorted(rows, key=lambda k: fr[f"pay_{k}"]) == sorted(rows, key=lambda k: fr[f"paper_{k}"]),
    }


def run4(tmp):
    """Run 4's outcome at cycle 500 under each reading: extinct, cooperators
    alone, defectors alone, or both, counted over the seeds."""
    counts = {}
    for label, (preset, changes) in READINGS.items():
        out = {"extinct": 0, "cooperators": 0, "defectors": 0, "both": 0}
        for seed in m.SEEDS:
            rows, _ = m.run(preset, seed, TICKS, tmp, changes)
            c, d = float(rows[-1]["cooperators"]), float(rows[-1]["defectors"])
            out["extinct" if c + d == 0 else "cooperators" if d == 0 else "defectors" if c == 0 else "both"] += 1
        counts[label] = out
    return counts


def verdicts(s, run4_counts):
    extinct = [run4_counts[k]["extinct"] for k in READINGS]
    montage = [
        ("neighbors: about a third keep helping, in clusters", 0.28 <= s["third_lo"] <= s["third_hi"] <= 0.38,
         f"Spatial games: helpers {s['third_lo']:.1%}–{s['third_hi']:.1%} (medians from 2, 10 and 25 % cheats)"),
        ("relatives: most help goes to kin next door", s["kin_help"] > 0.5,
         f"Ethnocentrism: {s['kin_help']:.1%} of helps go to relatives"),
        ("a shared look: most gifts go to an exact look-alike", s["twin_gifts"] > 0.5,
         f"Tags: {s['twin_gifts']:.1%} of gifts go to a Flump with exactly the giver's tag"),
        (f"a good name: in {s['everyone']:.0f} of 20 worlds, everyone ends up helping", True,
         "Reputation: every meeting ends in a gift by generation 1000"),
        ("a rule everyone enforces: the norm holds; how long depends on one sentence",
         s["meta"] >= 15 and s["his_est"] - s["long_est"] >= 8,
         f"Norms: established at generation 100 in {s['meta']:.0f} of 20; at 10⁶, {s['long_est']:.0f} of 20 "
         f"under Galán & Izquierdo's tie reading, {s['his_est']:.0f} under Axelrod's"),
        ("partners who stay: cooperation holds", s["frn_remain"] >= 0.9,
         f"Friends and strangers: fixed partners, payoff {s['frn']:.3f}, high {s['frn_remain']:.1%} of the time once "
         f"reached"),
        ("new strangers every period: it almost never takes hold", s["rwr"] < 1.2 and s["rwr_remain"] < 0.05,
         f"strangers: payoff {s['rwr']:.3f}, high {s['rwr_remain']:.1%} of the time once reached"),
    ]
    close = {
        "Nowak & May": abs((s["third_lo"] + s["third_hi"]) / 2 - 0.318) <= 0.02,
        "Huberman & Glance": s["hg_lost"] == 20,
        "Nowak, Bonhoeffer & May": min(s["nbm_async"], s["nbm_sync"]) > 0.5,
        "Hammond & Axelrod": abs(s["own"] - 0.763) <= 0.03,
        "Riolo, Cohen & Axelrod": abs(s["gifts"] - 0.736) <= 0.01,
        "Nowak & Sigmund": all(abs(s[k] - p) <= 0.07 for k, p in (("n20", 0.9), ("n50", 0.47), ("n100", 0.18))),
        "Axelrod": s["meta"] >= 15,
        "Cohen, Riolo & Axelrod": s["cra_worst"] <= 0.1 and s["cra_order"],
    }
    ledgers = [
        ("most of it reproduces, closely", all(close.values()),
         "the eight rows: " + ", ".join(f"{k} {'close' if v else 'NOT CLOSE'}" for k, v in close.items())),
        ("two results depend on a detail the papers give two ways",
         s["published"] < 760 and abs(s["working"] - 779) <= 15 and s["his_est"] - s["long_est"] >= 8,
         f"Epstein: {s['published']:.0f} under the book's rule, {s['working_only']:.0f} under the working paper's, "
         f"{s['working']:.0f} with founders starting with nothing (his 779); norms at 10⁶: {s['long_est']:.0f} and "
         f"{s['his_est']:.0f} of 20 established"),
        ("and two didn't come back, as written", s["seeing2_lo"] > 0.56 and min(extinct) >= EXTINCT,
         f"Hammond & Axelrod at double cost: {s['seeing2']:.1%} (every seed at least {s['seeing2_lo']:.1%}) against "
         f"56 %; Run 4 extinct by cycle 500 in " + ", ".join(f"{run4_counts[k]['extinct']} ({k})" for k in READINGS)
         + " of 20, cooperators alone in " + ", ".join(str(run4_counts[k]["cooperators"]) for k in READINGS)),
    ]
    point = [("why help a stranger? The Flumps do, when something makes the stranger less strange",
              all(h for _, h, _ in montage), "every montage claim holds")]
    return montage + ledgers + point


def pct(v):
    return f"{v:.0%}"


def shown(s, counts):
    """The ledgers' right-hand column, as the panel prints it."""
    extinct = [counts[k]["extinct"] for k in READINGS]
    return {
        "nm": f"{s['third_lo']:.1%}–{s['third_hi']:.1%}",
        "hg": f"in {s['hg_lost']:.0f} of 20",
        "nbm": f"{pct(s['nbm_async'])} and {pct(s['nbm_sync'])} help",
        "ha": f"{s['own']:.1%}",
        "rca": f"{s['gifts']:.1%}",
        "ns": f"{s['n20'] * 100:.0f}, {s['n50'] * 100:.0f}, {s['n100'] * 100:.0f}%",
        "ax": f"{s['meta']:.0f} of 20",
        "cra": f"each row within {s['cra_worst']:.2f}",
        "ep_book": f"{s['published']:.0f} helpers (his 779)",
        "ep_working": f"{s['working_only']:.0f}; {s['working']:.0f} if founders start with nothing",
        "ax_his": f"the norm holds at 10⁶ in {s['his_est']:.0f} of 20",
        "gi": f"in {s['long_est']:.0f} of 20",
        "ha2": f"{pct(s['seeing2'])}; every world above 56%",
        "run4": f"extinct in {min(extinct)}–{max(extinct)} of 20, either rule",
        "everyone": f"{s['everyone']:.0f}",
    }


def measure(tmp):
    s = sources()
    counts = run4(tmp)
    lines = ["## Epstein's Run 4 at cycle 500, under each reading (seeds 1–20)", "",
             "| reading | extinct | cooperators alone | defectors alone | both |", "|---|---|---|---|---|"]
    for k, c in counts.items():
        lines.append(f"| {READINGS[k][0]} | {c['extinct']} | {c['cooperators']} | {c['defectors']} | {c['both']} |")
    lines += ["", "## From the earlier episodes' measurements", ""]
    lines += [f"- {k}: {v:.4f}" if isinstance(v, float) else f"- {k}: {v}" for k, v in s.items()]
    return lines, verdicts(s, counts), {"medians": shown(s, counts), "seeds": 20}
