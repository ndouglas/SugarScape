"""Pure animation planning: everything the Blender handler sets on a frame is
a function of the frame, computed here and tested outside Blender.

Board space: one unit per cell, cell (0, 0) at the top-left (north-west), +X
east, +Y north; the felt rises `HEIGHT_PER_SUGAR` per unit of capacity.
Within a tick a moving Flump crouches until `CROUCH`, is airborne until
`LAND` and squashes on landing; the sugar it eats disappears as it lands.
Hunger (the droop) is 1 − sugar / 6, clamped: a display choice, not a rule.
"""

import math
from dataclasses import dataclass

HEIGHT_PER_SUGAR = 0.12
SPAWN_FRAMES = 12
POOF_FRAMES = 14
CROUCH, LAND = 0.15, 0.85


@dataclass(frozen=True)
class Timing:
    ticks_per_second: float
    start_tick: int = 0
    end_tick: int | None = None
    lead_in: float = 0.0
    fps: int = 30

    def frame(self, tick):
        """The (fractional) frame at which `tick` is reached; frames start at 1."""
        seconds = self.lead_in + (tick - self.start_tick) / self.ticks_per_second
        return 1 + seconds * self.fps

    def tick_at(self, frame):
        seconds = (frame - 1) / self.fps - self.lead_in
        tick = max(self.start_tick + seconds * self.ticks_per_second, self.start_tick)
        if self.end_tick is not None:
            tick = min(tick, self.end_tick)
        return tick


def corner_heights(capacity, w, h):
    """Each lattice corner's height: the mean capacity of the four cells
    around it (wrapping), so hills are smooth. Row-major, (w+1)·(h+1)."""
    out = []
    for cy in range(h + 1):
        for cx in range(w + 1):
            cells = [((cx + dx) % w, (cy + dy) % h) for dx in (-1, 0) for dy in (-1, 0)]
            out.append(sum(capacity[y * w + x] for x, y in cells) / 4 * HEIGHT_PER_SUGAR)
    return out


def cell_height(corners, x, y, w):
    row = w + 1
    i = y * row + x
    return (corners[i] + corners[i + 1] + corners[i + row] + corners[i + row + 1]) / 4


def cell_center(x, y, w, h):
    return (x - w / 2 + 0.5, h / 2 - y - 0.5)


def levels_at(d, tick):
    """Every site's sugar at a fractional tick: growback rises linearly
    through the tick; eaten sugar vanishes at the landing."""
    tick = min(max(tick, 0), d.ticks)
    lo = int(math.floor(tick))
    if lo >= d.ticks:
        return list(d.frames[d.ticks].sugar)
    a = tick - lo
    before, after = d.frames[lo].sugar, d.frames[lo + 1].sugar
    return [(b if a >= LAND else s) if b < s else s + (b - s) * a for s, b in zip(before, after)]


@dataclass(frozen=True)
class Pose:
    x: float
    y: float
    z: float
    sx: float
    sy: float
    sz: float
    yaw: float
    visible: bool
    sugar: float
    hunger: float


HIDDEN = Pose(0, 0, 0, 0, 0, 0, 0, False, 0, 0)


def smoothstep(u):
    u = min(max(u, 0.0), 1.0)
    return u * u * (3 - 2 * u)


def _squash(sz):
    """Scales for a height of `sz` with the volume kept."""
    s = 1 / math.sqrt(sz)
    return s, s, sz


def _step(a, b, n):
    """The shortest signed step from a to b on a ring of n."""
    d = b - a
    if abs(d) > n / 2:
        d -= math.copysign(n, d)
    return d


def _yaw(track, k, w, h):
    """Facing toward the latest move up to index k; 0 faces the camera (−Y)."""
    for i in range(k, 0, -1):
        (x0, y0), (x1, y1) = track.cells[i - 1], track.cells[i]
        if (x0, y0) != (x1, y1):
            dx, dy = _step(x0, x1, w), -_step(y0, y1, h)  # cell +y is south
            return math.atan2(dx, -dy)
    return 0.0


def _poof_start(track, timing):
    return timing.frame(track.death - 0.5)


def pose(track, timing, frame, corners, w, h):
    born = timing.frame(track.first)
    if frame < born - SPAWN_FRAMES:
        return HIDDEN
    if track.death is not None and frame > _poof_start(track, timing) + POOF_FRAMES:
        return HIDDEN
    tick = timing.tick_at(frame)
    last = len(track.cells) - 1
    k = min(max(int(math.floor(tick)) - track.first, 0), last)
    a = min(max(tick - (track.first + k), 0.0), 1.0) if k < last else 0.0
    here = track.cells[k]
    there = track.cells[k + 1] if k < last else here
    sugar = track.sugar[k] + (track.sugar[min(k + 1, last)] - track.sugar[k]) * a
    hunger = min(max(1 - sugar / 6, 0.0), 1.0)
    x0, y0 = cell_center(*here, w, h)
    x1, y1 = cell_center(*there, w, h)
    z0, z1 = cell_height(corners, *here, w), cell_height(corners, *there, w)
    x, y, z = x0, y0, z0
    sx = sy = sz = 1.0
    yaw = _yaw(track, k, w, h)
    if here != there:
        if abs(there[0] - here[0]) > w / 2 or abs(there[1] - here[1]) > h / 2:
            # A torus wrap: shrink away here and grow back there.
            sx = sy = sz = abs(1 - 2 * a)
            if a >= 0.5:
                x, y, z = x1, y1, z1
        elif a < CROUCH:
            sx, sy, sz = _squash(1 - 0.25 * math.sin(math.pi / 2 * a / CROUCH))
        elif a < LAND:
            u = (a - CROUCH) / (LAND - CROUCH)
            p = smoothstep(u)
            dist = math.hypot(x1 - x0, y1 - y0)
            x, y = x0 + (x1 - x0) * p, y0 + (y1 - y0) * p
            z = z0 + (z1 - z0) * p + (0.35 + 0.08 * dist) * 4 * u * (1 - u)
            sx, sy, sz = _squash(1 + 0.2 * math.sin(math.pi * u))
            yaw = _yaw(track, k + 1, w, h)
        else:
            x, y, z = x1, y1, z1
            sx, sy, sz = _squash(1 - 0.2 * math.sin(math.pi * (a - LAND) / (1 - LAND)))
            yaw = _yaw(track, k + 1, w, h)
    if frame < born:
        u = 1 - (born - frame) / SPAWN_FRAMES
        z += 1.5 * (1 - smoothstep(u))
        grow = smoothstep(u) * (1 + 0.15 * math.sin(math.pi * u))
        sx, sy, sz = sx * grow, sy * grow, sz * grow
    if track.death is not None and frame >= _poof_start(track, timing):
        u = (frame - _poof_start(track, timing)) / POOF_FRAMES
        s = (1 + 0.15 * math.sin(math.pi * min(u * 2, 1))) * (1 - smoothstep(u))
        sx, sy, sz = sx * s, sy * s, sz * s
    sz *= 1 - 0.12 * hunger
    return Pose(x, y, z, sx, sy, sz, yaw, True, sugar, hunger)


def blink(agent_id, frame):
    """Eye height: 1 open; a 6-frame blink every 3–6 s, phased by id."""
    period = 90 + (agent_id * 37) % 90
    phase = (frame + agent_id * 53) % period
    if phase < 6:
        return max(0.1, abs(phase - 3) / 3)
    return 1.0


def sight_cells(x, y, vision, w, h):
    """The cells an agent sees, as four rows (north, east, south, west), nearest first."""
    steps = [(0, -1), (1, 0), (0, 1), (-1, 0)]
    return [[((x + dx * k) % w, (y + dy * k) % h) for k in range(1, vision + 1)] for dx, dy in steps]


def histogram(values, bins, top):
    counts = [0] * bins
    for v in values:
        counts[min(max(int(v / top * bins), 0), bins - 1)] += 1
    return counts
