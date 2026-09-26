"""The felt board (smooth hills from capacity) and the sugar: one points
object with a `level` attribute, instanced as gumdrops by geometry nodes."""

import bmesh
import bpy

import animate
import seasons
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


def seasonal_felt(felt, d, timing):
    """Frosts the winter half of the board: the felt's material becomes a mix
    of felt and frost, per hemisphere, by the object's y (north is +y);
    returns an updater that sets each hemisphere's frost for a frame."""
    mat = bpy.data.materials.new("felt-seasons")
    mat.use_nodes = True
    nt = mat.node_tree
    nt.nodes.clear()
    out = nt.nodes.new("ShaderNodeOutputMaterial")
    grass = nt.nodes.new("ShaderNodeBsdfPrincipled")
    snow = nt.nodes.new("ShaderNodeBsdfPrincipled")
    for p, color, rough, sheen in ((grass, (0.30, 0.50, 0.26), 1.0, 0.8), (snow, (0.80, 0.87, 0.95), 0.55, 1.0)):
        p.inputs["Base Color"].default_value = (*color, 1)
        p.inputs["Roughness"].default_value = rough
        p.inputs["Sheen Weight"].default_value = sheen
    mix = nt.nodes.new("ShaderNodeMixShader")
    nt.links.new(grass.outputs[0], mix.inputs[1])
    nt.links.new(snow.outputs[0], mix.inputs[2])
    nt.links.new(mix.outputs[0], out.inputs["Surface"])
    # factor = south + (north − south) · [y > equator]
    coord, xyz = nt.nodes.new("ShaderNodeTexCoord"), nt.nodes.new("ShaderNodeSeparateXYZ")
    nt.links.new(coord.outputs["Object"], xyz.inputs[0])
    north = nt.nodes.new("ShaderNodeMath")
    north.operation = "GREATER_THAN"
    north.inputs[1].default_value = d.height / 2 - d.height // 2  # the equator's board y
    nt.links.new(xyz.outputs["Y"], north.inputs[0])
    frost_n, frost_s = nt.nodes.new("ShaderNodeValue"), nt.nodes.new("ShaderNodeValue")
    diff = nt.nodes.new("ShaderNodeMath")
    diff.operation = "SUBTRACT"
    nt.links.new(frost_n.outputs[0], diff.inputs[0])
    nt.links.new(frost_s.outputs[0], diff.inputs[1])
    scaled = nt.nodes.new("ShaderNodeMath")
    scaled.operation = "MULTIPLY"
    nt.links.new(diff.outputs[0], scaled.inputs[0])
    nt.links.new(north.outputs[0], scaled.inputs[1])
    factor = nt.nodes.new("ShaderNodeMath")
    factor.operation = "ADD"
    nt.links.new(scaled.outputs[0], factor.inputs[0])
    nt.links.new(frost_s.outputs[0], factor.inputs[1])
    nt.links.new(factor.outputs[0], mix.inputs[0])
    felt.data.materials.clear()
    felt.data.materials.append(mat)
    period = d.config["seasons"]["period"]

    def update(frame):
        # The step from frame k − 1 to k grows sugar with the season at tick
        # k − 1, which is floor(tick) between those frames: frost follows it.
        tick = timing.tick_at(frame)
        frost_n.outputs[0].default_value = seasons.frost(True, tick, period)
        frost_s.outputs[0].default_value = seasons.frost(False, tick, period)

    return update


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
