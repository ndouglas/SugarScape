"""The social-structure board: plain felt and 256 Flumps on a 16 × 16 grid,
each in the yarn of its friendliness p this period (see grid.py)."""

import bpy

import grid
from blender import flump, materials


def build(beat, d, timing):
    """The board and its Flumps, and their updater."""
    bpy.ops.mesh.primitive_plane_add(size=1)
    felt = bpy.context.active_object
    felt.name = "felt"
    side = grid.SIDE * grid.SPACING + 4
    felt.scale = (side, side, 1)
    felt.data.materials.append(materials.felt())
    for k in range(grid.SHADES):
        materials.YARN[f"friend{k}"] = grid.shade_color(k)
    protos = flump.crowd_prototypes(low=True)
    n = len(d.frames[0].agents)
    flumps = []
    for a in range(n):
        obj = flump.crowd_instance(f"agent{a}", protos["friend0"])
        obj.location = (*grid.position(a, d), 0)
        flumps.append(obj)

    def update(frame):
        f = d.frame(timing.tick_at(frame))
        for obj, agent in zip(flumps, f.agents):
            proto = protos[f"friend{grid.shade(agent[1])}"]
            if obj.instance_collection is not proto:
                obj.instance_collection = proto

    return update
