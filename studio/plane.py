"""The norms game on the board (see episodes/norms): an 8 × 8 plane of
temperaments, boldness from left (0) to right (7/7), vengefulness from front
(0) to back (7/7). Each Flump stands on its square, four abreast, crowding
with any others there. Galán & Izquierdo's regions (§5.12) are tinted:
"established" (mean boldness ≤ 2/7, vengefulness ≥ 5/7), back left, and
"collapsed" (boldness ≥ 6/7, vengefulness ≤ 1/7), front right; they are about
the population's means, so the tint marks where a population that has
established the norm (or lost it) mostly stands.

Each generation is a new list of offspring; the film keeps one Flump per
place in the list, and a Flump whose offspring lands on another square hops
there.
"""

import math
from dataclasses import dataclass

CELL = 4.4
ACROSS = 4
SPACING = 0.85


def square_center(b, v):
    return ((b - 3.5) * CELL, (v - 3.5) * CELL)


def established_square(b, v):
    return b <= 2 and v >= 5


def collapsed_square(b, v):
    return b >= 6 and v <= 1


def layout(frame):
    """Each place's (x, y): its square, filled four abreast in place order."""
    filled, out = {}, []
    for a in frame.agents:
        key = (a[0], a[1])
        n = filled.get(key, 0)
        filled[key] = n + 1
        row, col = divmod(n, ACROSS)
        cx, cy = square_center(*key)
        out.append((cx + (col - (ACROSS - 1) / 2) * SPACING, cy - CELL / 2 + 0.55 + row * SPACING))
    return out


@dataclass(frozen=True)
class Pose:
    x: float
    y: float
    z: float


def poses(d, t, hop, lift=1.0):
    """Every place's `Pose` at (fractional) frame t: between frames k and
    k + 1 a Flump whose square changes hops over the tick's last `hop` share;
    the rest slide to their new spots."""
    k = min(int(t), d.ticks)
    frac = 0.0 if k >= d.ticks else t - k
    now, nxt = d.frames[k], d.frames[min(k + 1, d.ticks)]
    here, there = layout(now), layout(nxt)
    start = 1 - hop
    u = min(max((frac - start) / hop, 0.0), 1.0) if hop > 0 else 1.0
    s = u * u * (3 - 2 * u)
    out = []
    for i, ((x0, y0), (x1, y1)) in enumerate(zip(here, there)):
        moved = now.agents[i][:2] != nxt.agents[i][:2]
        dist = math.hypot(x1 - x0, y1 - y0)
        z = lift * (0.3 + 0.06 * dist) * 4 * u * (1 - u) if moved else 0.0
        out.append(Pose(x0 + (x1 - x0) * s, y0 + (y1 - y0) * s, z))
    return out
