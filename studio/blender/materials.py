"""The handmade look: knit yarn, felt, gumdrops, glossy eyes; warm tabletop light."""

import math

import bpy

YARN = {
    "cream": (0.93, 0.86, 0.72),
    "coral": (0.95, 0.45, 0.38),
    "teal": (0.25, 0.62, 0.62),
    "lilac": (0.66, 0.55, 0.85),
    "butter": (0.98, 0.82, 0.40),
}


def _principled(name):
    """A material named `name` and its Principled BSDF, and whether it is new."""
    m = bpy.data.materials.get(name)
    fresh = m is None
    if fresh:
        m = bpy.data.materials.new(name)
        m.use_nodes = True
    return m, m.node_tree, m.node_tree.nodes["Principled BSDF"], fresh


def _bump(nt, p, scale, strength, wave=True):
    """A fine knit (wave bands plus noise) or felt (noise only) bump."""
    coord = nt.nodes.new("ShaderNodeTexCoord")
    noise = nt.nodes.new("ShaderNodeTexNoise")
    noise.inputs["Scale"].default_value = scale * 3
    nt.links.new(coord.outputs["Object"], noise.inputs["Vector"])
    height = noise.outputs["Fac"]
    if wave:
        w = nt.nodes.new("ShaderNodeTexWave")
        w.wave_type = "BANDS"
        w.bands_direction = "Z"
        w.inputs["Scale"].default_value = scale
        w.inputs["Distortion"].default_value = 3.0
        nt.links.new(coord.outputs["Object"], w.inputs["Vector"])
        add = nt.nodes.new("ShaderNodeMath")
        add.operation = "ADD"
        nt.links.new(w.outputs["Fac"], add.inputs[0])
        nt.links.new(noise.outputs["Fac"], add.inputs[1])
        height = add.outputs["Value"]
    bump = nt.nodes.new("ShaderNodeBump")
    bump.inputs["Strength"].default_value = strength
    nt.links.new(height, bump.inputs["Height"])
    nt.links.new(bump.outputs["Normal"], p.inputs["Normal"])


def knit(color_name):
    m, nt, p, fresh = _principled(f"knit-{color_name}")
    if fresh:
        p.inputs["Base Color"].default_value = (*YARN[color_name], 1)
        p.inputs["Roughness"].default_value = 0.85
        p.inputs["Sheen Weight"].default_value = 0.6
        p.inputs["Sheen Roughness"].default_value = 0.35
        p.inputs["Subsurface Weight"].default_value = 0.12
        _bump(nt, p, scale=28, strength=0.35)
    return m


def felt():
    m, nt, p, fresh = _principled("felt")
    if fresh:
        p.inputs["Base Color"].default_value = (0.30, 0.50, 0.26, 1)
        p.inputs["Roughness"].default_value = 1.0
        p.inputs["Sheen Weight"].default_value = 0.8
        _bump(nt, p, scale=6, strength=0.25, wave=False)
    return m


def gumdrop():
    m, nt, p, fresh = _principled("gumdrop")
    if fresh:
        p.inputs["Base Color"].default_value = (1.0, 0.72, 0.18, 1)
        p.inputs["Roughness"].default_value = 0.18
        p.inputs["Subsurface Weight"].default_value = 0.5
        p.inputs["Subsurface Radius"].default_value = (1.0, 0.6, 0.2)
        p.inputs["Coat Weight"].default_value = 1.0
    return m


def gloss(name, color, emission=0.0):
    m, nt, p, fresh = _principled(name)
    if fresh:
        p.inputs["Base Color"].default_value = (*color, 1)
        p.inputs["Roughness"].default_value = 0.05
        p.inputs["Coat Weight"].default_value = 1.0
        if emission:
            p.inputs["Emission Color"].default_value = (*color, 1)
            p.inputs["Emission Strength"].default_value = emission
    return m


def matte(name, color):
    m, nt, p, fresh = _principled(name)
    if fresh:
        p.inputs["Base Color"].default_value = (*color, 1)
        p.inputs["Roughness"].default_value = 1.0
    return m


def fading(name, color, strength=1.0):
    """An emissive material whose opacity the handler sets through its Mix
    Shader's factor: `material.node_tree.nodes["Mix"].inputs[0]`."""
    m = bpy.data.materials.get(name)
    if m is not None:
        return m
    m = bpy.data.materials.new(name)
    m.use_nodes = True
    nt = m.node_tree
    nt.nodes.clear()
    out = nt.nodes.new("ShaderNodeOutputMaterial")
    mix = nt.nodes.new("ShaderNodeMixShader")
    mix.name = "Mix"
    clear = nt.nodes.new("ShaderNodeBsdfTransparent")
    glow = nt.nodes.new("ShaderNodeEmission")
    glow.inputs["Color"].default_value = (*color, 1)
    glow.inputs["Strength"].default_value = strength
    nt.links.new(clear.outputs[0], mix.inputs[1])
    nt.links.new(glow.outputs[0], mix.inputs[2])
    nt.links.new(mix.outputs[0], out.inputs["Surface"])
    mix.inputs[0].default_value = 1.0
    return m


def lights_and_world(scene, extent):
    """A warm soft key, a cool fill and a warm rim (suns, so the look does not
    depend on the board's size), and a dim warm room."""
    world = bpy.data.worlds.new("room")
    world.use_nodes = True
    bg = world.node_tree.nodes["Background"]
    bg.inputs["Color"].default_value = (0.16, 0.13, 0.11, 1)
    bg.inputs["Strength"].default_value = 1.0
    scene.world = world
    for name, direction, strength, color, angle in [
        ("key", (-0.6, -0.8, 1.0), 2.4, (1.0, 0.88, 0.74), 12),
        ("fill", (0.9, -0.5, 0.45), 0.8, (0.75, 0.85, 1.0), 30),
        ("rim", (0.25, 1.0, 0.5), 2.0, (1.0, 0.8, 0.6), 6),
    ]:
        data = bpy.data.lights.new(name, "SUN")
        data.energy = strength
        data.color = color
        data.angle = math.radians(angle)
        obj = bpy.data.objects.new(name, data)
        obj.location = tuple(c * extent for c in direction)
        scene.collection.objects.link(obj)
        # A sun shines along its −Z: pointing +Z along `direction` lights
        # the board from that side.
        obj.rotation_euler = obj.location.to_track_quat("Z", "Y").to_euler()
