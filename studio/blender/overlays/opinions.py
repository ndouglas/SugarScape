"""Hegselmann & Krause's own picture beside the felt: opinion (up) against
period (across), a line per agent for a sample of them, colored by where
each started, drawn as far as the period shown."""

import dump
from blender import materials

from .panels import _polyline
from .parts import CREAM, box, card, text

SAMPLE = 120


def diagram(beat, d, ctx):
    """Top right: the opinion × time diagram. params: `periods` (the
    diagram's width in frames; default the shot's)."""
    anchor = ctx.screen.anchor("diagram", 0.72, 0.42)
    k = 0.7  # the panel's size on screen
    card("diagram-card", anchor, (0, 0, -0.01), (0.66 * k, 0.7 * k, 0.002))
    ink = materials.fading("diagram-ink", CREAM, 1.6)
    size_x, size_y, left, bottom = 0.56 * k, 0.52 * k, -0.28 * k, -0.29 * k
    text("diagram-y", "opinion", 0.032, ink, anchor, location=(left, bottom + size_y + 0.03, 0), align="LEFT")
    text("diagram-x", "period →", 0.032, ink, anchor, location=(left + size_x, bottom - 0.04, 0), align="RIGHT")
    box("diagram-axis-x", ink, anchor, location=(left + size_x / 2, bottom, 0.001), scale=(size_x, 0.003, 0.002))
    box("diagram-axis-y", ink, anchor, location=(left, bottom + size_y / 2, 0.001), scale=(0.003, size_y, 0.002))
    periods = beat.params.get("periods", d.ticks)
    inks = [materials.fading(f"diagram-{c}", materials.YARN[c], 2.5) for c in materials.START_YARN]
    n = len(d.placed)
    # Every (n / SAMPLE)th agent by starting rank: an even spread of starts.
    ranked = sorted(d.placed, key=d.start_rank.get)
    chosen = ranked[:: max(n // SAMPLE, 1)]
    lines = []
    for i in chosen:
        mat = inks[d.frames[0].groups[i] * len(inks) // dump.START_BINS]
        _, spline = _polyline(f"diagram-{i}", anchor, [(0, 0)] * (d.ticks + 1), mat)
        lines.append((i, spline))

    def at(k, i):
        x = left + size_x * min(k, periods) / max(periods, 1)
        return x, bottom + size_y * (d.frames[k].agents[i].x + 0.5) / d.width

    def update(frame):
        now = min(max(int(round(ctx.timing.tick_at(frame))), 0), d.ticks)
        for i, spline in lines:
            pts = []
            for k in range(d.ticks + 1):
                x, y = at(min(k, now), i)
                pts += [x, y, 0.002, 1]
            spline.points.foreach_set("co", pts)

    return update
