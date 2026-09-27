"""Families from a frame dump's births (rule S): who is whose child, which
generation each Flump belongs to, and its family line — the mother's, so a
line runs back to one founding mother. Pure; used by the Inheritance
episode's claims and its family colors."""

import statistics


def _births(d):
    for f in d.frames:
        for i, (sex, parents) in f.births.items():
            yield f.tick, i, sex, parents


def born_at(d):
    return {i: tick for tick, i, _, _ in _births(d)}


def children(d):
    out = {}
    for _, i, _, parents in _births(d):
        for p in parents or ():
            out.setdefault(p, []).append(i)
    return out


def generations(d):
    gen = {}
    for _, i, _, parents in _births(d):
        gen[i] = 0 if not parents else 1 + max(gen.get(p, 0) for p in parents)
    return gen


def families(d):
    """Each Flump's family line: its mother's, or its own if it has no parents."""
    sex, family = {}, {}
    for _, i, s, parents in _births(d):
        sex[i] = s
        mothers = [p for p in parents or () if sex.get(p) == "female"]
        family[i] = family.get(mothers[0], mothers[0]) if mothers else i
    return family


def bequests(d):
    """(tick, parent, sugar, heirs) for each death whose sugar passes on under
    rule I: the parent held sugar the tick before, and the heirs are its
    children alive after the tick — who split it equally, as the engine does.
    None without inheritance."""
    if not d.config.get("inheritance", {}).get("enabled"):
        return []
    kids = children(d)
    out = []
    for k in range(1, len(d.frames)):
        before, after = d.frames[k - 1].agents, d.frames[k].agents
        for i in d.frames[k].deaths:
            heirs = [c for c in kids.get(i, []) if c in after]
            held = before[i].sugar if i in before else 0
            if heirs and held > 0:
                out.append((d.frames[k].tick, i, held, heirs))
    return out


def _ranks(values):
    order = sorted(range(len(values)), key=lambda k: values[k])
    ranks = [0.0] * len(values)
    for rank, k in enumerate(order):
        ranks[k] = float(rank)
    return ranks


def spearman(xs, ys):
    """Spearman's rank correlation (ties broken by order)."""
    rx, ry = _ranks(xs), _ranks(ys)
    mx, my = statistics.mean(rx), statistics.mean(ry)
    num = sum((a - mx) * (b - my) for a, b in zip(rx, ry))
    den = (sum((a - mx) ** 2 for a in rx) * sum((b - my) ** 2 for b in ry)) ** 0.5
    return num / den if den else float("nan")
