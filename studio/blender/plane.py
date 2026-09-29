"""The norms board: an 8 × 8 plane of felt squares (see plane.py), Galán &
Izquierdo's "established" squares tinted green and "collapsed" squares red,
grid lines, axis and region labels, and one Flump per place in the
generation's list."""

import math

import bpy

import plane
from blender import flump, materials
from blender.overlays.parts import INK, text

GREEN, RED = (0.55, 0.85, 0.35), (0.85, 0.38, 0.3)


def _tile(name, x, y, size, material, z=0.004):
    bpy.ops.mesh.primitive_plane_add(size=1, location=(x, y, z))
    obj = bpy.context.active_object
    obj.name = name
    obj.scale = (size[0], size[1], 1)
    obj.data.materials.append(material)
    return obj


def _label(body, x, y, size, rotation=0.0):
    obj = text(f"label-{body}", body, size, materials.matte("plane-ink", INK), None, location=(x, y, 0.02))
    obj.rotation_euler = (0, 0, rotation)
    return obj


def board():
    side = 8 * plane.CELL
    _tile("felt", 0, 0, (side + 12, side + 12), materials.felt(), z=0.0)
    green, red = materials.matte("established", GREEN), materials.matte("collapsed", RED)
    for b in range(8):
        for v in range(8):
            x, y = plane.square_center(b, v)
            if plane.established_square(b, v):
                _tile(f"est{b}{v}", x, y, (plane.CELL, plane.CELL), green)
            elif plane.collapsed_square(b, v):
                _tile(f"col{b}{v}", x, y, (plane.CELL, plane.CELL), red)
    line = materials.matte("grid", (0.2, 0.3, 0.17))
    for i in range(9):
        c = (i - 4) * plane.CELL
        _tile(f"gx{i}", c, 0, (0.06, side), line, z=0.008)
        _tile(f"gy{i}", 0, c, (side, 0.06), line, z=0.008)
    _label("bolder →", -3, -side / 2 - 2.0, 2.0)
    _label("more vengeful →", -side / 2 - 2.0, 0, 2.0, rotation=math.pi / 2)
    # The region labels sit just outside the board beside their corners, where
    # no Flump stands on them.
    ex, _ = plane.square_center(1, 7)
    _label("norm established", ex, side / 2 + 1.6, 1.6)
    cx, _ = plane.square_center(6.5, 0)
    _label("norm collapsed", cx, -side / 2 - 2.0, 1.4)


def build(beat, d, timing):
    """The board and its 20 Flumps, and their updater."""
    board()
    protos = flump.crowd_prototypes()
    n = len(d.frames[0].agents)
    flumps = [flump.crowd_instance(f"place{k}", protos["cream"]) for k in range(n)]
    lift = min(1.0, max(0.0, (timing.tick_frames - 1) / 5))

    def update(frame):
        for obj, p in zip(flumps, plane.poses(d, timing.tick_at(frame), timing.hop, lift)):
            obj.location = (p.x, p.y, p.z)

    return update
