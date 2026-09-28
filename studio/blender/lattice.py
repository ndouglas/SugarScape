"""A spatial-games board: plain felt, one Flump on every square, blue for a
helper and red for a cheat. The felt under a Flump that just switched sides
glows in Nowak & May's change colors, through an image with a pixel per
square (see lattice.py for when)."""

import bpy

import animate
import lattice
from blender import flump, materials

HELPER, CHEAT = "blue", "red"
FELT = (0.30, 0.50, 0.26)
# Nowak & May's change colors: green for a cheat turned helper, yellow for a
# helper turned cheat; bright, so they read on the green felt.
GLOW = {"C": (0.45, 1.0, 0.3), "D": (1.0, 0.82, 0.12)}
GLOW_STRENGTH = 0.8


def _board(w, h):
    """The felt, and the image whose pixels color its squares (RGB) and set
    their glow (alpha)."""
    image = bpy.data.images.new("squares", w, h, alpha=True, float_buffer=True)
    m = bpy.data.materials.new("felt-squares")
    m.use_nodes = True
    nt = m.node_tree
    p = nt.nodes["Principled BSDF"]
    p.inputs["Roughness"].default_value = 1.0
    p.inputs["Sheen Weight"].default_value = 0.8
    tex = nt.nodes.new("ShaderNodeTexImage")
    tex.image = image
    tex.interpolation = "Closest"
    nt.links.new(tex.outputs["Color"], p.inputs["Base Color"])
    nt.links.new(tex.outputs["Color"], p.inputs["Emission Color"])
    glow = nt.nodes.new("ShaderNodeMath")
    glow.operation = "MULTIPLY"
    glow.inputs[1].default_value = GLOW_STRENGTH
    nt.links.new(tex.outputs["Alpha"], glow.inputs[0])
    nt.links.new(glow.outputs[0], p.inputs["Emission Strength"])
    materials._bump(nt, p, scale=6, strength=0.25, wave=False)
    bpy.ops.mesh.primitive_plane_add(size=1)
    felt = bpy.context.active_object
    felt.name = "felt"
    felt.scale = (w, h, 1)
    felt.data.materials.append(m)
    # A rim of plain felt around the squares.
    bpy.ops.mesh.primitive_plane_add(size=1, location=(0, 0, -0.01))
    rim = bpy.context.active_object
    rim.scale = (w + 2, h + 2, 1)
    rim.data.materials.append(materials.felt())
    return image


def build(beat, d, timing):
    """The board and its Flumps (rigs in close-ups, instances in crowds), and
    their updater."""
    w, h = d.width, d.height
    image = _board(w, h)
    pixels = [0.0] * (w * h * 4)
    cells = [i for i, c in enumerate(d.frames[0].strategies) if c != "."]
    spots = {i: lattice.position(i, w, h) for i in cells}
    rigs, instances, protos = {}, {}, None
    if beat.closeup:
        for i in cells:
            rigs[i] = flump.build_flump(f"square{i}", HELPER)
            rigs[i].root.location = (*spots[i], 0)
    else:
        # 9,801 Flumps a few pixels wide: low detail renders much faster.
        protos = flump.crowd_prototypes(low=True)
        for i in cells:
            instances[i] = flump.crowd_instance(f"square{i}", protos[HELPER])
            instances[i].location = (*spots[i], 0)

    def update(frame):
        m = lattice.moment(timing, frame, d.ticks)
        for i, s in enumerate(lattice.squares(d, m)):
            x, y = i % w, i // w
            p = ((h - 1 - y) * w + x) * 4
            if s.glow:
                g, k = GLOW[s.glow], s.strength
                pixels[p : p + 4] = [FELT[0] + (g[0] - FELT[0]) * k, FELT[1] + (g[1] - FELT[1]) * k,
                                     FELT[2] + (g[2] - FELT[2]) * k, k]
            else:
                pixels[p : p + 4] = [*FELT, 0.0]
            if i not in spots:
                continue
            color = HELPER if s.helper else CHEAT
            if beat.closeup:
                rig = rigs[i]
                rig.root.location.z = s.height
                flump.recolor(rig, color)
                rig.eyes.scale = (1, 1, animate.blink(i, frame))
            else:
                obj = instances[i]
                obj.location.z = s.height
                if obj.instance_collection is not protos[color]:
                    obj.instance_collection = protos[color]
        image.pixels.foreach_set(pixels)
        image.update()

    return update
