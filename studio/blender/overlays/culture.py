"""Axelrod's Fig. 1 on the felt: a lane between each pair of side
neighbors, darkest where they share nothing, lighter the more they share, and
gone where they are identical (one region)."""

import culture
from blender import materials

from .parts import INK, box


def lanes(beat, d, ctx):
    """The lanes of the step shown, recolored as neighbors converge."""
    f = len(next(iter(d.frames[0].traits.values())))
    felt = (0.72, 0.78, 0.62)
    # Level k (k features shared, 0 ≤ k < f): ink fading toward the felt.
    inks = [materials.matte(f"lane{k}", tuple(a + (b - a) * k / f for a, b in zip(INK, felt))) for k in range(f)]
    objs = []
    for i, j, x, y, along_y in culture.lanes(d.width, d.height):
        scale = (0.1, 0.96, 0.03) if along_y else (0.96, 0.1, 0.03)
        objs.append((i, j, box(f"lane-{i}-{j}", inks[0], None, location=(x, y, 0.02), scale=scale)))
    shown = [None]

    def update(frame):
        k = min(max(int(round(ctx.timing.tick_at(frame))), 0), d.ticks)
        if shown[0] == k:
            return
        shown[0] = k
        traits = d.frames[k].traits
        for i, j, obj in objs:
            s = culture.shared(traits, i, j)
            if s == f:
                obj.hide_render = True
            else:
                obj.hide_render = False
                obj.data.materials[0] = inks[s]

    return update
