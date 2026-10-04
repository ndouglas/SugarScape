"""Model-specific elderly Agent rigs; generic episode rigs are unchanged."""
from dataclasses import dataclass
import math
import bpy
from blender import flump, materials
from retirement_dance import age_style, dance_pose, join_pose
from retirement_visual import color

@dataclass
class CharacterRig:
    root: object
    body: object
    eyes: object
    yarn: list
    size: object
    upper: object
    feet: tuple
    foot_origins: tuple
    arms: tuple
    age_parts: list
    spectacles: list
    hero: bool
    origin: tuple
    join_material: object = None
    age_materials: object = None


def _stroke(name, points, radius, material, parent):
    curve = bpy.data.curves.new(name, 'CURVE')
    curve.dimensions = '3D'
    curve.bevel_depth = radius
    curve.bevel_resolution = 3
    line = curve.splines.new('POLY')
    line.points.add(len(points)-1)
    for p, xyz in zip(line.points, points):
        p.co = (*xyz, 1)
    obj = bpy.data.objects.new(name, curve)
    obj.parent = parent
    curve.materials.append(material)
    bpy.context.scene.collection.objects.link(obj)
    return obj


def build_character(name, member, hero=False, low=False):
    basic = flump.build_flump(name, color(member), low=low)
    size = basic.body.parent
    feet = tuple(part for part in basic.yarn if '.foot' in part.name)
    arms = tuple(part for part in basic.yarn if '.arm' in part.name)
    upper = flump._empty(name+'.upper', size, bpy.context.scene.collection)
    for part in list(size.children):
        if part is not upper and part not in feet:
            part.parent = upper
    age_parts, spectacles = [], []
    if not low:
        silver = bpy.data.materials.get('retirement-silver-yarn')
        if silver is None:
            silver = materials.knit('cream').copy()
            silver.name = 'retirement-silver-yarn'
            silver.node_tree.nodes['Principled BSDF'].inputs['Base Color'].default_value = (.65, .67, .69, 1)
        gray = materials.matte('retirement-gray-stitch', (.43, .46, .48))
        frame = materials.matte('retirement-spectacle-frame', (.22, .17, .13))
        for side in (-1, 1):
            age_parts.append(_stroke(name+'.silver-eyebrow',
                [(side*.12+dx, -.367, .64+.012*math.sin(i*math.pi/4))
                 for i, dx in enumerate((-.065, -.0325, 0, .0325, .065))], .013, silver, upper))
            for j in range(2):
                age_parts.append(_stroke(name+'.gray-stitch',
                    [(side*(.22+.017*k), -.318, .45-j*.04-.007*k) for k in range(3)], .004, gray, upper))
            # Open rings only, with no lenses: glossy pupils stay fully visible.
            ring = _stroke(name+'.spectacles',
                [(side*.15+.135*math.cos(k*2*math.pi/64), -.414,
                  .50+.135*math.sin(k*2*math.pi/64)) for k in range(65)], .006, frame, upper)
            spectacles.append(ring)
        spectacles.append(_stroke(name+'.spectacle-bridge',
                                  [(-.015, -.414, .50), (0, -.426, .513), (.015, -.414, .50)],
                                  .005, frame, upper))
    rig = CharacterRig(basic.root, basic.body, basic.eyes, basic.yarn, size, upper,
                       feet, tuple(tuple(p.location) for p in feet), arms,
                       age_parts, spectacles, hero, (0., 0., 0.))
    pose_character(rig, member, 0)
    return rig


def pose_character(rig, member, seconds, dancing=None, dance_amount=None, gaze=0., response=0., age_amount=1.):
    """Apply local dance about rig.origin. Caller owns root scale/yaw and blink.

    Only teaching before/after may override recorded retirement via dancing.
    Appearance refreshes for birth replacement without retaining elder cues.
    """
    style = age_style(member, rig.hero)
    rig.root['slot'], rig.root['born'], rig.root['age'] = member['id'], member['born'], member['age']
    if rig.age_materials is None:
        rig.age_materials = {}
        for part in rig.age_parts + rig.spectacles:
            original = part.data.materials[0]
            if original.name not in rig.age_materials:
                private = original.copy()
                private.name = rig.root.name + '.age-' + original.name
                nodes = private.node_tree.nodes
                output = next(n for n in nodes if n.type == 'OUTPUT_MATERIAL')
                shader = output.inputs['Surface'].links[0].from_socket
                clear = nodes.new('ShaderNodeBsdfTransparent')
                mix = nodes.new('ShaderNodeMixShader')
                mix.name = 'Age opacity'
                private.node_tree.links.new(clear.outputs[0],mix.inputs[1])
                private.node_tree.links.new(shader,mix.inputs[2])
                private.node_tree.links.new(mix.outputs[0],output.inputs['Surface'])
                rig.age_materials[original.name] = private
            part.data.materials[0] = rig.age_materials[original.name]
    for private in rig.age_materials.values():
        private.node_tree.nodes['Age opacity'].inputs[0].default_value = age_amount
    for part in rig.age_parts:
        part.hide_render = part.hide_viewport = not style.elderly or age_amount <= 0
    for part in rig.spectacles:
        part.hide_render = part.hide_viewport = not style.spectacles or age_amount <= 0
    rig.root['retired'] = member['retired']
    if dance_amount is None:
        flump.recolor(rig, color(member))
    else:
        # One private material per joining rig; shared friend yarn stays untouched.
        if rig.join_material is None:
            rig.join_material = materials.knit(color(member)).copy()
            rig.join_material.name = rig.root.name + '.joining-yarn'
        amount = max(0., min(1., dance_amount)) if member['retired'] else 0.
        worker = materials.YARN[color(dict(member, retired=False))]
        retiree = materials.YARN['cream']
        rgba = tuple(a+(b-a)*amount for a,b in zip(worker, retiree)) + (1.,)
        rig.join_material.node_tree.nodes['Principled BSDF'].inputs['Base Color'].default_value = rgba
        for part in rig.yarn:
            part.data.materials[0] = rig.join_material
    actual = member if dancing is None else dict(member, retired=dancing)
    pose = (dance_pose(seconds, actual) if dance_amount is None
            else join_pose(seconds, actual, dance_amount))
    rig.root.location = tuple(a+b for a,b in zip(rig.origin, pose.offset))
    rig.size.rotation_euler = (0., 0., pose.yaw)
    rig.upper.rotation_euler = (0., pose.lean + response, gaze)
    for foot, origin, offset in zip(rig.feet, rig.foot_origins, (pose.left_foot, pose.right_foot)):
        foot.location = tuple(a+b for a,b in zip(origin, offset))
    for index, arm in enumerate(rig.arms):
        arm.rotation_euler.y = (-1 if index == 0 else 1) * pose.arm_swing
    return pose
