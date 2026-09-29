"""The tags board: a disc of felt, a wheel of shades just inside the ring
(so the eye can read which shade stands where), and one Flump per place in
the population's list, in the yarn of its tag's shade (see ring.py for
where each stands and when it leaps)."""

import math

import bpy

import ring
from blender import flump, materials

WHEEL_SEGMENTS = 96


def _wheel():
    """A thin band of shades just inside the ring, from tag 0 round to 1."""
    inner, outer = ring.RADIUS - 1.5, ring.RADIUS - 0.9
    for s in range(WHEEL_SEGMENTS):
        t0, t1 = s / WHEEL_SEGMENTS, (s + 1) / WHEEL_SEGMENTS
        verts = []
        for t in (t0, t1):
            a = ring.angle(t)
            verts += [(inner * math.cos(a), inner * math.sin(a), 0.01), (outer * math.cos(a), outer * math.sin(a), 0.01)]
        mesh = bpy.data.meshes.new(f"wheel{s}")
        mesh.from_pydata(verts, [], [(0, 1, 3, 2)])
        mesh.materials.append(materials.matte(f"shade-{ring.hue((t0 + t1) / 2)}", ring.shade(ring.hue((t0 + t1) / 2))))
        obj = bpy.data.objects.new(f"wheel{s}", mesh)
        bpy.context.scene.collection.objects.link(obj)


def build(beat, d, timing):
    """The board and its 100 Flumps, and their updater."""
    bpy.ops.mesh.primitive_circle_add(vertices=128, radius=ring.RADIUS + 9, fill_type="NGON")
    felt = bpy.context.active_object
    felt.name = "felt"
    felt.data.materials.append(materials.felt())
    _wheel()
    for k in range(ring.HUES):
        materials.YARN[f"hue-{k}"] = ring.shade(k)
    protos = flump.crowd_prototypes()
    n = len(d.frames[0].agents)
    flumps = [flump.crowd_instance(f"place{k}", protos[f"hue-{ring.hue(d.frames[0].agents[k].tag)}"]) for k in range(n)]
    # Leaps only when a generation lasts long enough to see them.
    lift = min(1.0, max(0.0, (timing.tick_frames - 1) / 5))

    def update(frame):
        for k, p in enumerate(ring.poses(d, timing.tick_at(frame), timing.hop, lift)):
            obj = flumps[k]
            obj.location = (p.x, p.y, p.z)
            proto = protos[f"hue-{ring.hue(p.tag)}"]
            if obj.instance_collection is not proto:
                obj.instance_collection = proto

    return update
