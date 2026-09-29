"""Overlays for the image-scoring street: each meeting of the generation
shown, as an arc from donor to recipient, yarn for a gift and red for a
refusal."""

import math

import street
from blender import materials

from .ring import ARC_POINTS, _curve


def meetings(beat, d, ctx):
    """The next generation's meetings as they play out over the tick, after
    its hops (see street.shown), each arc staying until the tick's end."""
    most = max((len(f.meetings) for f in d.frames), default=0)
    gift, refusal = materials.knit("butter"), materials.knit("red")
    arcs = [(_curve(f"gift{i}", gift, ARC_POINTS), _curve(f"refusal{i}", refusal, ARC_POINTS)) for i in range(most)]
    for _, r in arcs:
        r.data.bevel_depth = 0.025

    def update(frame):
        t = ctx.timing.tick_at(frame)
        k = min(int(t), d.ticks)
        f = d.frames[min(k + 1, d.ticks)]
        spots = street.layout(f)
        count = street.shown(d, t, ctx.timing.hop)
        for i, (g, r) in enumerate(arcs):
            for obj, on in ((g, i < count and f.meetings[i][2]), (r, i < count and not f.meetings[i][2])):
                pts = obj.data.splines[0].points
                if not on:
                    for p in pts:
                        p.co = (0, 0, -20, 1)
                    continue
                (x0, y0), (x1, y1) = spots[f.meetings[i][0]], spots[f.meetings[i][1]]
                height = 0.8 + 0.1 * math.hypot(x1 - x0, y1 - y0)
                for j, p in enumerate(pts):
                    u = j / (ARC_POINTS - 1)
                    p.co = (x0 + (x1 - x0) * u, y0 + (y1 - y0) * u, 0.6 + height * 4 * u * (1 - u), 1)

    return update
