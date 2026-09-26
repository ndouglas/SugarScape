"""Builds one beat's scene and drives it with a single frame-change handler."""

from types import SimpleNamespace

import bpy
from mathutils import Vector

import animate
import camera as cam
import dump as dump_mod
import lineage
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
    # Screen-space panels float just in front of the lens and would blur.
    data.dof.use_dof = beat.closeup and not any(name in overlays.SCREEN for name in beat.overlays)
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
    yarn colors in order), collection instances in crowds — and their updater.
    With params colors="family", a Flump wears its family line's color (its
    mother's, back to a founding mother); with colors="tribe", its tribe's,
    changing as its tribe does."""
    w, h = d.width, d.height
    colors = list(materials.CROWD_YARN)
    RIGS.clear()
    instances = {}
    mode = beat.params.get("colors")
    family = lineage.families(d) if mode == "family" else {}
    order = {id_: i for i, id_ in enumerate(d.placed)}

    def base_color(id_):
        key = family.get(id_, id_)
        return colors[order.get(key, key) % len(colors)]

    def tribe_color(id_, frame):
        f = d.frames[min(max(int(round(timing.tick_at(frame))), 0), d.ticks)]
        g = f.groups.get(id_)
        return materials.TRIBE_YARN[g] if g is not None else None

    protos = None if beat.closeup else flump.crowd_prototypes()
    for id_ in tracks:
        start = tribe_color(id_, timing.frame(tracks[id_].first)) if mode == "tribe" else None
        color = start or base_color(id_)
        if beat.closeup:
            RIGS[id_] = flump.build_flump(f"flump{id_}", color)
        else:
            instances[id_] = flump.crowd_instance(f"flump{id_}", protos[color])

    def update(frame):
        for id_, t in tracks.items():
            p = animate.pose(t, timing, frame, corners, w, h)
            color = tribe_color(id_, frame) if mode == "tribe" and p.visible else None
            if beat.closeup:
                rig = RIGS[id_]
                flump.apply(rig.root, p)
                rig.eyes.scale = (1, 1, animate.blink(id_, frame) * (1 - 0.4 * p.hunger))
                if color:
                    flump.recolor(rig, color)
            else:
                flump.apply(instances[id_], p)
                if color and instances[id_].instance_collection is not protos[color]:
                    instances[id_].instance_collection = protos[color]

    return update


def build_beat(beat, d, preview, compare=None, measured=None):
    """Builds the beat's scene; returns its per-frame updaters."""
    scene = bpy.context.scene
    reset(scene, preview)
    scene.frame_start, scene.frame_end = 1, beat.frames
    # Motion blur smears into ghosts once Flumps hop more than once a frame:
    # it fades out as the beat speeds past one tick a frame.
    ticks_per_frame = beat.ticks_per_second / 30
    scene.render.motion_blur_shutter = 0.3 / max(1.0, ticks_per_frame)
    updaters = []
    timing = corners = None
    tracks = {}
    if d is not None:
        timing = beat.timing(d.ticks)
        felt, corners = board.felt_board(d)
        if d.config["seasons"]["enabled"]:
            updaters.append(board.seasonal_felt(felt, d, timing))
        elif any(any(f.pollution) for f in d.frames):
            updaters.append(board.sooty_felt(felt, d, timing))
        _, update_sugar = board.sugar(d, corners, timing)
        updaters.append(update_sugar)
        # Only the Flumps alive during this beat's ticks get objects: a long
        # run with births can have many thousands over its whole length.
        first, last = timing.tick_at(1), timing.tick_at(beat.frames + 1)
        tracks = {i: t for i, t in dump_mod.tracks(d).items() if animate.alive_in(t, first, last)}
        updaters.append(_agents(beat, d, tracks, timing, corners))
        materials.lights_and_world(scene, max(d.width, d.height))
    else:
        updaters.append(_title_card())
        materials.lights_and_world(scene, 12)
    camera_obj, update_camera = add_camera(scene, beat)
    screen = overlays.Screen(camera_obj)
    # The camera and its screen anchors move first; overlays read them.
    updaters[:0] = [update_camera, screen.update]
    ctx = SimpleNamespace(
        camera=camera_obj, screen=screen, timing=timing, tracks=tracks, corners=corners, rigs=RIGS, compare=compare,
        measured=measured or {},
    )
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
