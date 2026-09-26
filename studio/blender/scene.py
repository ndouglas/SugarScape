"""Builds one beat's scene and drives it with a single frame-change handler."""

from types import SimpleNamespace

import bpy
from mathutils import Vector

import animate
import camera as cam
import dump as dump_mod
from blender import board, flump, materials, overlays

UPDATERS = []
# The first exception the frame handler raised, if any.
ERRORS = []
# The close-up rigs by agent id, for overlays.
RIGS = {}


def reset(scene, preview):
    for obj in list(bpy.data.objects):
        bpy.data.objects.remove(obj)
    scene.render.engine = "BLENDER_EEVEE"
    # The frame handler edits scene data while rendering: without the lock,
    # Blender can deadlock mid-animation.
    scene.render.use_lock_interface = True
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
    # Toy-like depth of field in close-ups; wide shots stay sharp, and so do
    # their screen-space displays.
    data.dof.use_dof = beat.closeup
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
                flump.apply(rig.root, p)
                rig.eyes.scale = (1, 1, animate.blink(id_, frame) * (1 - 0.4 * p.hunger))
            else:
                flump.apply(instances[id_], p)

    return update


def _title_card():
    """A beat with no shot: a felt tabletop and one Flump, blinking at the
    viewer. Nothing is simulated, so it does nothing else."""
    bpy.ops.mesh.primitive_plane_add(size=1)
    felt = bpy.context.active_object
    felt.scale = (16, 10, 1)
    felt.data.materials.append(materials.felt())
    rig = flump.build_flump("host", "cream")

    def update(frame):
        rig.eyes.scale = (1, 1, animate.blink(7, frame))

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
        updaters.append(_title_card())
        materials.lights_and_world(scene, 12)
    camera_obj, update_camera = add_camera(scene, beat)
    screen = overlays.Screen(camera_obj)
    # The camera and its screen anchors move first; overlays read them.
    updaters[:0] = [update_camera, screen.update]
    ctx = SimpleNamespace(camera=camera_obj, screen=screen, timing=timing, tracks=tracks, corners=corners, rigs=RIGS)
    for name in beat.overlays:
        updaters.append(overlays.BUILDERS[name](beat, d, ctx))
    return updaters


def install(updaters):
    """Registers the one frame-change handler. Blender prints a handler's
    exceptions and renders on with stale poses, so the first one is kept
    in ERRORS for render.py to fail on."""
    UPDATERS[:] = updaters
    ERRORS.clear()

    def on_frame(scene, depsgraph=None):
        frame = scene.frame_current + scene.frame_subframe
        try:
            for update in UPDATERS:
                update(frame)
        except Exception as e:
            if not ERRORS:
                ERRORS.append(f"frame {frame}: {e!r}")
            raise

    bpy.app.handlers.frame_change_pre.clear()
    bpy.app.handlers.frame_change_pre.append(on_frame)
    on_frame(bpy.context.scene)
