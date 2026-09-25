"""Builds one beat's scene and drives it with a single frame-change handler."""

import bpy
from mathutils import Vector

import animate
import camera as cam
import dump as dump_mod
from blender import board, flump, materials

UPDATERS = []
# The close-up rigs by agent id, for overlays.
RIGS = {}


def reset(scene, preview):
    for obj in list(bpy.data.objects):
        bpy.data.objects.remove(obj)
    scene.render.engine = "BLENDER_EEVEE"
    scene.render.fps = 30
    scene.render.resolution_x, scene.render.resolution_y = (960, 540) if preview else (1920, 1080)
    scene.render.resolution_percentage = 100
    scene.render.use_motion_blur = True
    scene.render.motion_blur_shutter = 0.3
    scene.eevee.taa_render_samples = 16 if preview else 96
    scene.eevee.use_raytracing = not preview
    scene.view_settings.view_transform = "AgX"
    scene.view_settings.look = "AgX - Medium High Contrast"
    scene.view_settings.exposure = -0.6
    scene.render.image_settings.file_format = "PNG"


def add_camera(scene, beat):
    data = bpy.data.cameras.new("camera")
    data.dof.use_dof = True
    data.dof.aperture_fstop = 4.0
    obj = bpy.data.objects.new("camera", data)
    scene.collection.objects.link(obj)
    scene.camera = obj

    def update(frame):
        eye, target, lens = cam.camera_at(beat.camera, (frame - 1) / 30)
        obj.location = eye
        direction = Vector(target) - Vector(eye)
        obj.rotation_euler = direction.to_track_quat("-Z", "Y").to_euler()
        data.lens = lens
        data.dof.focus_distance = direction.length

    return obj, update


def _agents(beat, d, tracks, timing, corners):
    """Flumps for every agent — full rigs in close-ups (placed agents take the
    yarn colors in order), collection instances in crowds — and their updater."""
    w, h = d.width, d.height
    colors = list(materials.YARN)
    RIGS.clear()
    instances = {}
    if beat.closeup:
        order = {id_: i for i, id_ in enumerate(d.placed)}
        for id_ in tracks:
            color = colors[order.get(id_, id_) % len(colors)]
            RIGS[id_] = flump.build_flump(f"flump{id_}", color)
    else:
        protos = flump.crowd_prototypes()
        for id_ in tracks:
            instances[id_] = flump.crowd_instance(f"flump{id_}", protos[id_ % len(protos)])

    def update(frame):
        for id_, t in tracks.items():
            p = animate.pose(t, timing, frame, corners, w, h)
            if beat.closeup:
                rig = RIGS[id_]
                flump.apply(rig.root, p, rig.parts)
                rig.eyes.scale = (1, 1, animate.blink(id_, frame) * (1 - 0.4 * p.hunger))
            else:
                flump.apply(instances[id_], p)

    return update


def build_beat(beat, d, preview):
    """Builds the beat's scene; returns its per-frame updaters."""
    scene = bpy.context.scene
    reset(scene, preview)
    scene.frame_start, scene.frame_end = 1, beat.frames
    updaters = []
    timing = corners = None
    tracks = {}
    if d is not None:
        timing = beat.timing(d.ticks)
        _, corners = board.felt_board(d)
        _, update_sugar = board.sugar(d, corners, timing)
        updaters.append(update_sugar)
        tracks = dump_mod.tracks(d)
        updaters.append(_agents(beat, d, tracks, timing, corners))
        materials.lights_and_world(scene, max(d.width, d.height))
    else:
        materials.lights_and_world(scene, 12)
    camera_obj, update_camera = add_camera(scene, beat)
    updaters.append(update_camera)
    return updaters


def install(updaters):
    UPDATERS[:] = updaters

    def on_frame(scene, depsgraph=None):
        frame = scene.frame_current + scene.frame_subframe
        for update in UPDATERS:
            update(frame)

    bpy.app.handlers.frame_change_pre.clear()
    bpy.app.handlers.frame_change_pre.append(on_frame)
    on_frame(bpy.context.scene)
