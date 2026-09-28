"""The tags model on the board (see episodes/tags): a ring of felt where
each Flump stands at the point its tag names, in the yarn of that shade.

Tags run clockwise from just right of the top (0) to just left of it (1),
leaving a gap at the top: in the model tags don't wrap around, so 0 and 1
are as far apart as two shades can be, not neighbors. Flumps with the same exact
tag crowd together in a pile packed outward from the ring, so a cluster of
twins is a visible crowd. The model has no lasting individuals: each
generation is a new list of offspring. The film keeps one Flump per place in
the list: when place k's offspring descends from place k's agent it stays
(or shuffles within its pile); when it copies its opponent, it leaps across
the ring to its role model's shade.
"""

import colorsys
import math
from dataclasses import dataclass

RADIUS = 14.0
SPACING = 0.95
HUES = 24
# The share of the circle left open at the top, between tags 1 and 0.
GAP = 0.08


def angle(tag):
    """The ring's angle for a tag: clockwise from just right of the top
    (tag 0) to just left of it (tag 1)."""
    return math.pi / 2 - (GAP / 2 + tag * (1 - GAP)) * 2 * math.pi


def on_ring(tag, r=RADIUS):
    a = angle(tag)
    return (r * math.cos(a), r * math.sin(a))


def hue(tag):
    """The shade bucket (0 … HUES − 1) of a tag."""
    return min(int(tag * HUES), HUES - 1)


def shade(k):
    """Bucket k's yarn, as linear RGB."""
    r, g, b = colorsys.hsv_to_rgb((k + 0.5) / HUES, 0.55, 0.9)
    return (r ** 2.2, g ** 2.2, b ** 2.2)


def _hex(n):
    """The first n points of a hexagonal packing around (0, 0), nearest
    first, one unit apart."""
    pts = [(0.0, 0.0)]
    ring = 1
    while len(pts) < n:
        corners = [(math.cos(math.pi / 3 * i), math.sin(math.pi / 3 * i)) for i in range(6)]
        for i in range(6):
            (x0, y0), (x1, y1) = corners[i], corners[(i + 1) % 6]
            for step in range(ring):
                t = step / ring
                pts.append((ring * (x0 + (x1 - x0) * t), ring * (y0 + (y1 - y0) * t)))
        ring += 1
    return pts[:n]


def layout(frame):
    """Each place's (x, y) on the board: piles of identical tags, each
    centered just outside the ring at its tag's angle, the pile's members in
    place order from its center outward."""
    groups = {}
    for k, a in enumerate(frame.agents):
        groups.setdefault(a.tag, []).append(k)
    out = [None] * len(frame.agents)
    for tag, places in groups.items():
        n = len(places)
        extent = SPACING * math.sqrt(n) * 0.55
        cx, cy = on_ring(tag, RADIUS + extent)
        for k, (dx, dy) in zip(places, _hex(n)):
            out[k] = (cx + dx * SPACING, cy + dy * SPACING)
    return out


def leaps(before, after):
    """Places whose offspring copied another place's agent (so it leaps),
    rather than its own place's."""
    return [a.parent != b.id for b, a in zip(before.agents, after.agents)]


@dataclass(frozen=True)
class Pose:
    x: float
    y: float
    z: float
    tag: float


def poses(d, t, hop, lift=1.0):
    """Every place's `Pose` at (fractional) tick t: between generations k
    and k + 1 a Flump that stays slides to its new spot in the pile, and one
    that copies another leaps in an arc over the tick's last `hop` share,
    taking its new shade at the top."""
    k = min(int(t), d.ticks)
    frac = 0.0 if k >= d.ticks else t - k
    now = d.frames[k]
    nxt = d.frames[k + 1] if k < d.ticks else now
    here, there = layout(now), layout(nxt)
    jump = leaps(now, nxt) if nxt is not now else [False] * len(now.agents)
    start = 1 - hop
    u = min(max((frac - start) / hop, 0.0), 1.0) if hop > 0 else 1.0
    s = u * u * (3 - 2 * u)
    out = []
    for i, ((x0, y0), (x1, y1)) in enumerate(zip(here, there)):
        x, y = x0 + (x1 - x0) * s, y0 + (y1 - y0) * s
        dist = math.hypot(x1 - x0, y1 - y0)
        z = lift * (0.3 + 0.06 * dist) * 4 * u * (1 - u) if jump[i] or dist > 1.5 else 0.0
        tag = nxt.agents[i].tag if u >= 0.5 else now.agents[i].tag
        out.append(Pose(x, y, z, tag))
    return out
