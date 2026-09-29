"""Image scoring on the board (see episodes/image): a street of twelve
columns, one per threshold k, from "help everyone" (k = −5) on the left to
"help no one" (k = +6) on the right. Each Flump stands in its threshold's
column, three abreast, in the yarn of its score (red −5, pale 0, green +5).

Each generation is a new list of offspring; the film keeps one Flump per
place in the list. When place i's offspring takes another threshold, its
Flump hops to that column; otherwise it slides to its new spot in its own.
"""

import colorsys
import math
from dataclasses import dataclass

K_MIN, K_MAX = -5, 6
COLUMN = 3.6  # between columns' centers
ACROSS = 3
SPACING = 0.95
SCORES = range(-5, 6)


def column_x(k):
    return (k - (K_MIN + K_MAX) / 2) * COLUMN


def layout(frame):
    """Each place's (x, y): its threshold's column, filled three abreast from
    the front row back, in place order."""
    filled = {}
    out = []
    for a in frame.agents:
        k = a[1] if a[1] is not None else K_MAX
        n = filled.get(k, 0)
        filled[k] = n + 1
        row, col = divmod(n, ACROSS)
        out.append((column_x(k) + (col - (ACROSS - 1) / 2) * SPACING, row * SPACING))
    return out


def score_color(score):
    """A score's yarn, as linear RGB: red at −5, a neutral pale at 0, green
    at +5, more saturated the further from 0."""
    h = 0.33 if score > 0 else 0.0
    r, g, b = colorsys.hsv_to_rgb(h, 0.3 + 0.5 * abs(score) / 5 if score else 0.0, 0.92)
    return (r ** 2.2, g ** 2.2, b ** 2.2)


@dataclass(frozen=True)
class Pose:
    x: float
    y: float
    z: float
    score: int


def replay(meetings, count, n):
    """Every place's score after the first `count` meetings of a generation:
    scores start at 0 and move one up for a gift, one down for a refusal,
    within ±5."""
    scores = [0] * n
    for donor, _, helped in meetings[:count]:
        scores[donor] = max(-5, min(5, scores[donor] + (1 if helped else -1)))
    return scores


def shown(d, t, hop):
    """How many of the next generation's meetings have played by (fractional)
    tick t: they play out after the tick's first `hop` share (the hops)."""
    k = min(int(t), d.ticks)
    if k >= d.ticks:
        return 0
    frac = t - k
    return int(len(d.frames[k + 1].meetings) * min(max((frac - hop) / max(1 - hop, 1e-9), 0.0), 1.0))


def poses(d, t, hop, lift=1.0):
    """Every place's `Pose` at (fractional) tick t. Between generations k and
    k + 1, as the model does: first the next generation takes its places (a
    Flump changing threshold hops to its new column over the tick's first
    `hop` share; the rest slide), then it plays. With the meetings recorded,
    they play out one by one over the rest of the tick and each Flump's score
    follows them from 0; without, the score turns from generation k's to
    k + 1's at the top of the hop."""
    k = min(int(t), d.ticks)
    frac = 0.0 if k >= d.ticks else t - k
    now = d.frames[k]
    nxt = d.frames[k + 1] if k < d.ticks else now
    here, there = layout(now), layout(nxt)
    u = min(frac / hop, 1.0) if hop > 0 else 1.0
    s = u * u * (3 - 2 * u)
    if nxt is not now and nxt.meetings:
        scores = replay(nxt.meetings, shown(d, t, hop), len(nxt.agents)) if u >= 0.5 else [a[2] for a in now.agents]
    else:
        scores = [(b if u >= 0.5 else a)[2] for a, b in zip(now.agents, nxt.agents)]
    out = []
    for i, ((x0, y0), (x1, y1)) in enumerate(zip(here, there)):
        moved = now.agents[i][1] != nxt.agents[i][1]
        dist = math.hypot(x1 - x0, y1 - y0)
        z = lift * (0.3 + 0.08 * dist) * 4 * u * (1 - u) if moved else 0.0
        out.append(Pose(x0 + (x1 - x0) * s, y0 + (y1 - y0) * s, z, scores[i]))
    return out
