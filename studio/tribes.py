"""Tribes (rule K, Animations III-6/III-7) read from a frame: how alike
neighbours are, and which tribe leads where. Pure; used by the Tribes
episode's claims and its gauge."""


def neighbours_alike(f, w, h):
    """The share of von Neumann-adjacent pairs of agents in the same tribe
    (the board wraps); NaN with no adjacent pairs."""
    at = {(a.x, a.y): f.groups[i] for i, a in f.agents.items()}
    same = total = 0
    for (x, y), g in at.items():
        for dx, dy in ((1, 0), (0, 1)):
            other = at.get(((x + dx) % w, (y + dy) % h))
            if other is not None:
                total += 1
                same += other == g
    return same / total if total else float("nan")


def majority(f, where=lambda a: True):
    """The leading tribe among the agents `where` selects, and its share
    (None, None when there are none). Ties go to tribe 0."""
    groups = [f.groups[i] for i, a in f.agents.items() if where(a)]
    if not groups:
        return None, None
    counts = {}
    for g in groups:
        counts[g] = counts.get(g, 0) + 1
    lead = min(counts, key=lambda g: (-counts[g], g))
    return lead, counts[lead] / len(groups)
