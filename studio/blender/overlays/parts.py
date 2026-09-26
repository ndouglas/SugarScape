"""The pieces overlays are made of: screen anchors, text, boxes, balls,
cards and gauges, the palette, and a followed Flump's pose."""

import pathlib

import bmesh
import bpy

import animate
from blender import materials


FONT = pathlib.Path(__file__).resolve().parents[2] / "fonts" / "Baloo2.ttf"


CREAM, INK = (1.0, 0.96, 0.88), (0.06, 0.05, 0.04)


SENSOR = 36.0  # mm, horizontal (Blender's default sensor fit for 16:9)


# How far in front of the lens screen-space things float: nearer than
# anything in the scene. (Beats with them have no depth of field.)
DEPTH = 0.4


class Screen:
    """Anchors glued to the camera's view: x in [-1, 1] across the frame's
    width, y in [-1, 1] up its height."""

    def __init__(self, camera_obj):
        self.camera = camera_obj
        self.anchors = []

    def anchor(self, name, x, y, depth=DEPTH):
        e = bpy.data.objects.new(name, None)
        e.parent = self.camera
        bpy.context.scene.collection.objects.link(e)
        self.anchors.append((e, x, y, depth))
        return e

    def update(self, frame):
        for e, x, y, depth in self.anchors:
            half = depth * SENSOR / 2 / self.camera.data.lens
            e.location = (x * half, y * half * 9 / 16, -depth)
            e.scale = (half, half, half)


def _font():
    return bpy.data.fonts.load(str(FONT), check_existing=True)


def text(name, body, size, material, parent, location=(0, 0, 0), align="CENTER"):
    cu = bpy.data.curves.new(name, type="FONT")
    cu.body = body
    cu.font = _font()
    cu.size = size
    cu.align_x = align
    cu.align_y = "CENTER"
    cu.materials.append(material)
    obj = bpy.data.objects.new(name, cu)
    obj.parent = parent
    obj.location = location
    bpy.context.scene.collection.objects.link(obj)
    return obj


def box(name, material, parent, location=(0, 0, 0), scale=(1, 1, 1)):
    mesh = bpy.data.meshes.new(name)
    bm = bmesh.new()
    bmesh.ops.create_cube(bm, size=1.0)
    bm.to_mesh(mesh)
    bm.free()
    mesh.materials.append(material)
    obj = bpy.data.objects.new(name, mesh)
    obj.parent = parent
    obj.location, obj.scale = location, scale
    bpy.context.scene.collection.objects.link(obj)
    bev = obj.modifiers.new("round", "BEVEL")
    bev.width, bev.segments = 0.02, 3
    return obj


def ball(name, radius, material, parent=None):
    mesh = bpy.data.meshes.new(name)
    bm = bmesh.new()
    bmesh.ops.create_uvsphere(bm, u_segments=12, v_segments=6, radius=radius)
    bm.to_mesh(mesh)
    bm.free()
    mesh.materials.append(material)
    obj = bpy.data.objects.new(name, mesh)
    obj.parent = parent
    bpy.context.scene.collection.objects.link(obj)
    return obj


def card(name, parent, location, scale):
    """A dark card behind a display, so it reads on any shot. Its bevel is
    wide, so small cards come out as pills."""
    obj = box(name, materials.matte("card", (0.03, 0.025, 0.02)), parent, location=location, scale=scale)
    obj.modifiers["round"].width = 0.08
    obj.modifiers["round"].segments = 6
    return obj


def gauge(screen, name, color, x, y):
    anchor = screen.anchor(name, x, y)
    box(f"{name}-back", materials.knit("cream"), anchor, scale=(0.5, 0.05, 0.004))
    fill = box(f"{name}-fill", materials.knit(color), anchor, location=(0, -0.004, 0.003), scale=(0.46, 0.035, 0.004))
    label = text(f"{name}-text", "", 0.045, materials.fading(f"{name}-ink", CREAM, 1.2), anchor, location=(-0.25, 0.06, 0), align="LEFT")
    return fill, label


def flump_pose(ctx, d, id_, frame):
    return animate.pose(ctx.tracks[id_], ctx.timing, frame, ctx.corners, d.width, d.height)


def turn_to_camera(obj, ctx):
    direction = ctx.camera.matrix_world.translation - obj.location
    obj.rotation_euler = direction.to_track_quat("Z", "Y").to_euler()


SUMMER, WINTER = (1.0, 0.72, 0.25), (0.62, 0.8, 1.0)
