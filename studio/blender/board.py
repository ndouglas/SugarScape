"""The felt board (smooth hills from capacity) and the sugar: one points
object with a `level` attribute, instanced as gumdrops by geometry nodes."""

import bmesh
import bpy

import animate
from blender import materials

GUMDROP_SIZE = 0.45


def felt_board(d):
    """The felt mesh, its corners' heights, and nothing else: cells are 1 unit."""
    w, h = d.width, d.height
    corners = animate.corner_heights(d.capacity, w, h)
    verts = [
        (cx - w / 2, h / 2 - cy, corners[cy * (w + 1) + cx])
        for cy in range(h + 1)
        for cx in range(w + 1)
    ]
    faces = [
        (cy * (w + 1) + cx, cy * (w + 1) + cx + 1, (cy + 1) * (w + 1) + cx + 1, (cy + 1) * (w + 1) + cx)
        for cy in range(h)
        for cx in range(w)
    ]
    mesh = bpy.data.meshes.new("felt")
    mesh.from_pydata(verts, [], faces)
    for poly in mesh.polygons:
        poly.use_smooth = True
    obj = bpy.data.objects.new("felt", mesh)
    obj.data.materials.append(materials.felt())
    sub = obj.modifiers.new("smooth", "SUBSURF")
    sub.levels = sub.render_levels = 2
    bpy.context.scene.collection.objects.link(obj)
    return obj, corners


def _gumdrop_prototype():
    mesh = bpy.data.meshes.new("gumdrop")
    bm = bmesh.new()
    bmesh.ops.create_uvsphere(bm, u_segments=24, v_segments=12, radius=0.5)
    bm.to_mesh(mesh)
    bm.free()
    for poly in mesh.polygons:
        poly.use_smooth = True
    mesh.materials.append(materials.gumdrop())
    g = bpy.data.objects.new("gumdrop", mesh)
    g.scale = (1, 1, 0.8)
    bpy.context.scene.collection.objects.link(g)
    g.hide_render = True
    g.hide_viewport = True
    return g


def _instancer_tree(proto):
    ng = bpy.data.node_groups.new("gumdrops", "GeometryNodeTree")
    ng.interface.new_socket("Geometry", in_out="INPUT", socket_type="NodeSocketGeometry")
    ng.interface.new_socket("Geometry", in_out="OUTPUT", socket_type="NodeSocketGeometry")
    n = ng.nodes
    gin, gout = n.new("NodeGroupInput"), n.new("NodeGroupOutput")
    to_points = n.new("GeometryNodeMeshToPoints")
    info = n.new("GeometryNodeObjectInfo")
    info.inputs["Object"].default_value = proto
    info.inputs["As Instance"].default_value = True
    level = n.new("GeometryNodeInputNamedAttribute")
    level.data_type = "FLOAT"
    level.inputs["Name"].default_value = "level"
    # Scale ∝ √level: a full site (4) is GUMDROP_SIZE across, one of 1 half that.
    root = n.new("ShaderNodeMath")
    root.operation = "POWER"
    root.inputs[1].default_value = 0.5
    scale = n.new("ShaderNodeMath")
    scale.operation = "MULTIPLY"
    scale.inputs[1].default_value = GUMDROP_SIZE / 2
    inst = n.new("GeometryNodeInstanceOnPoints")
    link = ng.links.new
    link(gin.outputs[0], to_points.inputs["Mesh"])
    link(to_points.outputs[0], inst.inputs["Points"])
    link(info.outputs["Geometry"], inst.inputs["Instance"])
    link(level.outputs["Attribute"], root.inputs[0])
    link(root.outputs[0], scale.inputs[0])
    link(scale.outputs[0], inst.inputs["Scale"])
    link(inst.outputs[0], gout.inputs[0])
    return ng


def sugar(d, corners, timing):
    """The gumdrops, and an updater that sets each site's level for a frame."""
    w, h = d.width, d.height
    verts = []
    for y in range(h):
        for x in range(w):
            cx, cy = animate.cell_center(x, y, w, h)
            verts.append((cx, cy, animate.cell_height(corners, x, y, w) + 0.05))
    mesh = bpy.data.meshes.new("sugar")
    mesh.from_pydata(verts, [], [])
    mesh.attributes.new("level", "FLOAT", "POINT")
    obj = bpy.data.objects.new("sugar", mesh)
    bpy.context.scene.collection.objects.link(obj)
    mod = obj.modifiers.new("gumdrops", "NODES")
    mod.node_group = _instancer_tree(_gumdrop_prototype())

    def update(frame):
        levels = animate.levels_at(d, timing.tick_at(frame), timing.hop)
        mesh.attributes["level"].data.foreach_set("value", levels)
        mesh.update()

    return obj, update
