"""Axelrod's culture map as the studio draws it (see episodes/culture): the
lanes between side-by-side sites, where his Fig. 1 draws a boundary darker
the less two neighbors share. Board coordinates: one unit per site, site
(x, y) at (x − w/2 + 0.5, h/2 − y − 0.5); site id = y · w + x."""


def lanes(w, h):
    """Every pair of side neighbors on a bounded w × h map as (i, j, x, y,
    along_y): the two sites, the middle of their shared edge, and whether the
    lane runs north–south (between left and right neighbors)."""
    out = []
    for y in range(h):
        for x in range(w - 1):
            out.append((y * w + x, y * w + x + 1, x + 1 - w / 2, h / 2 - y - 0.5, True))
    for y in range(h - 1):
        for x in range(w):
            out.append((y * w + x, (y + 1) * w + x, x - w / 2 + 0.5, h / 2 - y - 1, False))
    return out


def shared(traits, i, j):
    """How many features sites i and j hold in common."""
    return sum(a == b for a, b in zip(traits[i], traits[j]))
