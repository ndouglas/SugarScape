"""The image-scoring board: felt, a label under each threshold's column
(and "help everyone" and "help no one" at the two ends), and one Flump per
place in the population's list in the yarn of its score (see street.py for
where each stands and when it hops)."""

import bpy

import street
from blender import flump, materials
from blender.overlays.parts import INK, text


def _label(body, x, y, size):
    obj = text(f"label-{body}", body, size, materials.matte("label-ink", INK), None, location=(x, y, 0.02))
    return obj


def build(beat, d, timing):
    """The board and its Flumps, and their updater."""
    width = street.column_x(street.K_MAX) - street.column_x(street.K_MIN) + 2 * street.COLUMN
    bpy.ops.mesh.primitive_plane_add(size=1, location=(0, 15, 0))
    felt = bpy.context.active_object
    felt.name = "felt"
    felt.scale = (width, 44, 1)
    felt.data.materials.append(materials.felt())
    for k in range(street.K_MIN, street.K_MAX + 1):
        _label(f"{k:+d}" if k else "0", street.column_x(k), -1.8, 1.3)
    _label("help everyone", street.column_x(street.K_MIN) + 1.2, -3.6, 1.1)
    _label("help no one", street.column_x(street.K_MAX) - 1.0, -3.6, 1.1)
    for score in street.SCORES:
        materials.YARN[f"score{score}"] = street.score_color(score)
    protos = flump.crowd_prototypes()
    n = len(d.frames[0].agents)
    flumps = [flump.crowd_instance(f"place{i}", protos["score0"]) for i in range(n)]
    lift = min(1.0, max(0.0, (timing.tick_frames - 1) / 5))

    def update(frame):
        for obj, p in zip(flumps, street.poses(d, timing.tick_at(frame), timing.hop, lift)):
            obj.location = (p.x, p.y, p.z)
            proto = protos[f"score{p.score}"]
            if obj.instance_collection is not proto:
                obj.instance_collection = proto

    return update
