"""Following the Crowd, episode 5, "Listening to the like-minded": Hegselmann
& Krause's bounded confidence (2002) and Lorenz (2006), each captioned claim
measured over 20 seeds, 50 where the paper used 50 (see studio/measure.py and
docs/superpowers/specs/2026-09-30-opinions-spike.md). Rules follow the survey's
`opinions.*` (survey/src/claims/opinions.rs).

Runs go until stable (at most 5,000 periods; the lattice 20,000).
"""

import concurrent.futures
import json
import subprocess

import measure as m
from measure import median

CAP, LATTICE_CAP = 5_000, 20_000
MIDDLE_SEEDS = range(1, 51)
GRID = (0.1, 0.15, 0.2, 0.25, 0.3)
LEAN = {"confidence": "asymmetric", "epsilon_left": 0.02, "epsilon_right": 0.2}
EXTREMES = {"bias": 1.0, "epsilon": 0.4}


def _last(preset, seed, tmp, changes=None, ticks=CAP):
    rows, agents = m.run(preset, seed, ticks, tmp, changes)
    return {k: float(v) for k, v in rows[-1].items()}, agents


def _camps(agents, big=0.05):
    """Distinct final opinions (within 1e-6) as (mean, share), and whether
    one holding `big` or more sits in the middle (0.4–0.6)."""
    xs = sorted(float(a["opinion"]) for a in agents)
    groups = [[xs[0]]]
    for v in xs[1:]:
        (groups[-1].append(v) if v - groups[-1][-1] < 1e-6 else groups.append([v]))
    camps = [(sum(g) / len(g), len(g) / len(xs)) for g in groups]
    return len(camps), any(0.4 < mu < 0.6 and share >= big for mu, share in camps)


def seed_row(tmp, seed):
    out = {}
    out["few"] = _last("hk-plurality", seed, tmp)[0]["clusters"]
    out["many"] = _last("hk-consensus", seed, tmp)[0]["clusters"]
    out["lean_mean"] = _last("hk-asym-b", seed, tmp, LEAN)[0]["mean_opinion"]
    out["extremes_range"] = _last("hk-bias", seed, tmp, EXTREMES)[0]["range"]
    # The survey's grid: ε 0.1–0.3 and both neighborhoods on the torus,
    # against everyone in view at the same ε.
    polar, everyone, opinions = 0, 0, []
    for e in GRID:
        for hood in ("moore", "von_neumann"):
            lat, _ = _last("hk-lattice", seed, tmp, {"epsilon": e, "lattice.neighborhood": hood}, ticks=LATTICE_CAP)
            polar += lat["second"] >= 0.2
            opinions.append(lat["clusters"])
        all_, _ = _last("hk-lattice", seed, tmp, {"epsilon": e, "interaction": "all"})
        everyone += all_["second"] >= 0.2
    out["lattice_polar"], out["everyone_polar"] = float(polar), float(everyone)
    out["lattice_clusters"] = sorted(opinions)[len(opinions) // 2]
    for n in (50, 1000):
        s, _ = _last("hk-consensus", seed, tmp, {"epsilon": 0.22, "agents": n})
        out[f"agree_{n}"] = float(s["clusters"] == 1)
    return out


def middle_row(tmp, seed):
    stats, agents = _last("hk-polarisation", seed, tmp)
    n, mid = _camps(agents)
    return {"two": float(n == 2), "mid": float(mid), "stable": stats["stable_at"]}


def edges(tmp, eps=0.2):
    """Fig. 4's regular 50 at ε 0.2 after one period: whether the outermost
    moved inward while everyone ε or more from both ends stayed put."""
    src, out = tmp / "edges.json", tmp / "edges.frames.json"
    src.write_text(json.dumps({"preset": "hk-regular-50", "ticks": 1}))
    subprocess.run([m.CLI, "shot", src, "--out", out], check=True, stdout=subprocess.DEVNULL)
    raw = json.loads(out.read_text())
    order = sorted(range(raw["agents"]), key=lambda i: raw["frames"][0]["opinions"][i])
    before = [raw["frames"][0]["opinions"][i] for i in order]
    after = [raw["frames"][1]["opinions"][i] for i in order]
    lo, hi = before[0], before[-1]
    centre = [k for k, x in enumerate(before) if x - lo >= eps and hi - x >= eps]
    still = all(abs(after[k] - before[k]) < 1e-12 for k in centre)
    return after[0] > before[0] and after[-1] < before[-1] and still and len(centre) > 0


def verdicts(rows, middle, config, survey):
    n = len(rows)
    md = {k: median(rows, k) for k in next(iter(rows.values()))}
    two = sum(r["two"] for r in middle.values())
    mid = sum(r["mid"] for r in middle.values())
    nm = len(middle)
    return [
        ("625 Flumps, each with an opinion between 0 and 1", config["crowd"] == (625, "random"),
         "625 agents, opinions drawn uniformly"),
        ("each listens only to those close enough, and moves to their average",
         config["rule"] == ("symmetric", "simultaneous", "all"),
         "symmetric confidence, everyone updating at once, all agents in view"),
        ("listen to very few, and dozens of opinions survive", md["few"] >= 24,
         f"ε 0.01: median {md['few']:.1f} opinions survive (theirs: \"exactly 38\")"),
        ("listen widely, and everyone agrees", all(r["many"] == 1 for r in rows.values()),
         f"ε 0.25: {sum(r['many'] == 1 for r in rows.values())} of {n} end in one opinion"),
        ("in between, Hegselmann and Krause's run ends in two camps", config["middle_seed_two"],
         "their Fig. 2b at ε 0.15; the shot's seed ends in exactly two camps here"),
        ("here that happens about a third of the time; more often, a third camp holds the middle",
         0.2 <= two / nm <= 0.45 and mid > two,
         f"ε 0.15, {nm} runs: {two:.0f} end in exactly two opinions, {mid:.0f} keep a middle camp of 5 % or more"),
        ("camps form at the edges first: those at the ends hear only one side, and move in",
         config["edges"],
         "Fig. 4 (50 evenly spaced, ε 0.2), period 1: the outermost move inward, everyone ε or more from both ends "
         "stays put"),
        ("listen more to one side, and the whole crowd drifts there", md["lean_mean"] >= 0.8,
         f"listening 0.2 to the right and 0.02 to the left: median final mean opinion {md['lean_mean']:.2f}"),
        ("let each lean toward its own side, and the camps are pushed to the ends",
         all(r["extremes_range"] >= 0.95 for r in rows.values()),
         f"bias m = 1, ε 0.4: final range at least 0.95 in "
         f"{sum(r['extremes_range'] >= 0.95 for r in rows.values())} of {n} (theirs: \"0 and 1\")"),
        ("hear only your neighbors on a map, and two camps become rare: one crowd, with stranded minorities",
         sum(r["lattice_polar"] for r in rows.values()) <= 0.1 * 10 * n
         and sum(r["everyone_polar"] for r in rows.values()) >= 0.4 * 5 * n and md["lattice_clusters"] >= 10,
         f"25 × 25 torus, ε 0.1–0.3, both neighborhoods: a second camp of a fifth in "
         f"{sum(r['lattice_polar'] for r in rows.values()):.0f} of {10 * n} runs, against "
         f"{sum(r['everyone_polar'] for r in rows.values()):.0f} of {5 * n} with everyone in view; median "
         f"{md['lattice_clusters']:.0f} distinct opinions"),
        ("and numbers matter: a big crowd agrees where a small one splits",
         sum(r["agree_1000"] for r in rows.values()) > sum(r["agree_50"] for r in rows.values()),
         f"ε 0.22: consensus in {sum(r['agree_1000'] for r in rows.values()):.0f} of {n} with 1,000 agents, "
         f"{sum(r['agree_50'] for r in rows.values()):.0f} with 50 (Lorenz 2006)"),
        ("nearly all they reported, the Flumps reproduce", survey["ok"], survey["why"]),
        ("who you'll listen to decides whether a crowd agrees, splits, or shatters",
         md["many"] == 1 and md["few"] >= 24 and mid + two >= 0.9 * nm,
         f"ε 0.25 agrees, 0.15 splits ({two + mid:.0f} of {nm} into two or three camps), 0.01 shatters "
         f"({md['few']:.0f} opinions)"),
    ]


def _survey():
    path = m.REPO / "survey" / "out" / "results-opinions.json"
    results = json.loads(path.read_text())
    failing = [r["id"] for r in results if r["verdict"] != "Holds"]
    ok = len(results) >= 15 and failing == ["opinions.fig-2b.two-camps"]
    return {"ok": ok, "why": f"survey opinions.*: {len(results) - len(failing)} of {len(results)} hold; "
                             f"the one that does not: {', '.join(failing) or 'none'}"}


def measure(tmp):
    c = m.preset_config("hk-polarisation", tmp)
    for p in ("hk-plurality", "hk-consensus", "hk-asym-b", "hk-bias", "hk-lattice", "hk-regular-50"):
        m.preset_config(p, tmp)
    with concurrent.futures.ThreadPoolExecutor(8) as pool:
        rows = dict(zip(m.SEEDS, pool.map(lambda s: seed_row(tmp, s), m.SEEDS)))
        middle = dict(zip(MIDDLE_SEEDS, pool.map(lambda s: middle_row(tmp, s), MIDDLE_SEEDS)))
    two_seeds = [s for s, r in middle.items() if r["two"]]
    config = {"crowd": (c["agents"], c["start"]), "rule": (c["confidence"], c["updating"], c["interaction"]),
              "edges": edges(tmp), "middle_seed_two": MIDDLE_SHOT_SEED in two_seeds}
    keys = list(next(iter(rows.values())))
    lines = ["## Bounded confidence", ""] + m.table(rows, keys)
    lines += ["", f"ε 0.15 over {len(middle)} seeds: exactly two opinions in seeds {two_seeds}.",
              "", "Typical seeds: few " + str(m.typical_seed(rows, ["few"])) + ", lean "
              + str(m.typical_seed(rows, ["lean_mean"])) + ", extremes " + str(m.typical_seed(rows, ["extremes_range"]))
              + ", lattice " + str(m.typical_seed(rows, ["lattice_clusters"])) + "."]
    return lines, verdicts(rows, middle, config, _survey()), {"medians": {k: median(rows, k) for k in keys},
                                                             "seeds": len(rows)}


# The two-camp seed the "middle" beat films (one of those listed in measurements.md).
MIDDLE_SHOT_SEED = 8
