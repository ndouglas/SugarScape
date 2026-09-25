"""The Flump: a crocheted blob with nubbin limbs and glossy eyes, built from
primitives. The root empty sits at the feet, so scaling it squashes toward
the ground; the eyes' empty blinks. Front faces −Y."""

from dataclasses import dataclass

import bmesh
import bpy

from blender import materials

SIZE = 0.9
BLUSH = (1.0, 0.55, 0.6)


@dataclass
class FlumpRig:
    root: object
    body: object
    eyes: object
    parts: list


def _sphere(name, radius, location, scale, material, parent, collection, segments=32):
    mesh = bpy.data.meshes.new(name)
    bm = bmesh.new()
    bmesh.ops.create_uvsphere(bm, u_segments=segments, v_segments=segments // 2, radius=radius)
    bm.to_mesh(mesh)
    bm.free()
    for poly in mesh.polygons:
        poly.use_smooth = True
    mesh.materials.append(material)
    obj = bpy.data.objects.new(name, mesh)
    obj.location, obj.scale = location, scale
    obj.parent = parent
    collection.objects.link(obj)
    return obj


def _empty(name, parent, collection, location=(0, 0, 0)):
    e = bpy.data.objects.new(name, None)
    e.location = location
    e.parent = parent
    collection.objects.link(e)
    return e


def build_flump(name, color, collection=None):
    collection = collection or bpy.context.scene.collection
    parts = []
    root = _empty(name, None, collection)
    size = _empty(f"{name}.size", root, collection)
    size.scale = (SIZE,) * 3
    yarn = materials.knit(color)

    def sphere(*args, **kwargs):
        obj = _sphere(*args, **kwargs)
        parts.append(obj)
        return obj

    body = sphere(f"{name}.body", 0.4, (0, 0, 0.38), (1, 0.95, 0.9), yarn, size, collection)
    sub = body.modifiers.new("smooth", "SUBSURF")
    sub.levels = sub.render_levels = 1
    blush = materials.matte("blush", BLUSH)
    for side in (-1, 1):
        sphere(f"{name}.arm", 0.1, (side * 0.38, 0, 0.4), (1, 1, 1), yarn, size, collection, segments=16)
        sphere(f"{name}.foot", 0.1, (side * 0.16, -0.06, 0.06), (1, 1.3, 0.6), yarn, size, collection, segments=16)
        sphere(f"{name}.blush", 0.06, (side * 0.24, -0.33, 0.37), (1, 0.3, 0.7), blush, size, collection, segments=16)
    eyes = _empty(f"{name}.eyes", size, collection, location=(0, -0.35, 0.5))
    ink = materials.gloss("eye", (0.01, 0.01, 0.012))
    shine = materials.gloss("shine", (1, 1, 1), emission=4.0)
    for side in (-1, 1):
        eye = sphere(f"{name}.eye", 0.09, (side * 0.12, 0, 0), (0.72, 0.5, 1.35), ink, eyes, collection, segments=24)
        # The highlight's scale undoes the eye's, so it stays round.
        sphere(f"{name}.shine", 0.026, (-0.028, -0.07, 0.035), (1 / 0.72, 1 / 0.5, 1 / 1.35), shine, eye, collection, segments=12)
    return FlumpRig(root, body, eyes, [root, size, eyes, *parts])


def crowd_prototypes():
    """One Flump per yarn color, each in its own collection that is never
    linked to the scene, so only its instances render."""
    protos = []
    for color in materials.YARN:
        coll = bpy.data.collections.new(f"flump-{color}")
        build_flump(f"proto-{color}", color, coll)
        protos.append(coll)
    return protos


def crowd_instance(name, collection):
    e = bpy.data.objects.new(name, None)
    e.instance_type = "COLLECTION"
    e.instance_collection = collection
    bpy.context.scene.collection.objects.link(e)
    return e


def apply(obj, pose, parts=()):
    """Poses a rig's root or a crowd instance; hides `parts` with it."""
    for o in (obj, *parts):
        o.hide_render = not pose.visible
    if not pose.visible:
        return
    obj.location = (pose.x, pose.y, pose.z)
    obj.rotation_euler = (0, 0, pose.yaw)
    obj.scale = (pose.sx, pose.sy, pose.sz)
