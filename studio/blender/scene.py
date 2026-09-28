"""Builds one beat's scene and drives it with a single frame-change handler."""

from types import SimpleNamespace

import bpy
from mathutils import Vector

import animate
import camera as cam
import dump as dump_mod
import lineage
import ring
from blender import board, flump, materials, overlays
from blender import lattice as lattice_board
from blender import ring as ring_board

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
    changing as its tribe does; with colors="sick", sickly green while it
    carries a disease and its own color when well; with colors="strategy"
    (the demographic PD), blue for a helper and red for a cheat; with
    colors="tag" (ethnocentrism), its tag's yarn."""
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

    live = mode in ("tribe", "sick", "strategy", "tag")

    def live_color(id_, frame):
        """The color the tick shown gives a Flump (None: its own)."""
        f = d.frames[min(max(int(round(timing.tick_at(frame))), 0), d.ticks)]
        if mode == "tag":
            g = f.groups.get(id_)
            if g is None:
                g = d.frames[tracks[id_].first].groups.get(id_)
            return materials.TAG_YARN[g % len(materials.TAG_YARN)] if g is not None else None
        if mode in ("tribe", "strategy"):
            g = f.groups.get(id_)
            if g is None and mode == "strategy":
                # A newborn pops in just before its cycle's frame; a
                # strategy never changes, so its first frame's will do.
                g = d.frames[tracks[id_].first].groups.get(id_)
            return materials.TRIBE_YARN[g] if g is not None else None
        return "sick" if f.diseases.get(id_) else None

    protos = None if beat.closeup else flump.crowd_prototypes()
    for id_ in tracks:
        start = live_color(id_, timing.frame(tracks[id_].first)) if live else None
        color = start or base_color(id_)
        if beat.closeup:
            RIGS[id_] = flump.build_flump(f"flump{id_}", color)
        else:
            instances[id_] = flump.crowd_instance(f"flump{id_}", protos[color])

    def update(frame):
        for id_, t in tracks.items():
            p = animate.pose(t, timing, frame, corners, w, h)
            color = (live_color(id_, frame) or base_color(id_)) if live and p.visible else None
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
    if isinstance(d, dump_mod.Ring):
        timing = beat.timing(d.ticks)
        updaters.append(ring_board.build(beat, d, timing))
        materials.lights_and_world(scene, 2 * ring.RADIUS)
    elif isinstance(d, dump_mod.Lattice):
        timing = beat.timing(d.ticks)
        updaters.append(lattice_board.build(beat, d, timing))
        materials.lights_and_world(scene, max(d.width, d.height))
    elif d is not None:
        timing = beat.timing(d.ticks)
        if d.model == "ethno":
            # Flat felt whose squares show each Flump's kind, and a Flump per
            # square rather than per Flump (see lattice_board.ethno).
            corners = animate.corner_heights(animate.relief(d), d.width, d.height)
            updaters.append(lattice_board.ethno(beat, d, timing))
        else:
            felt, corners = board.felt_board(d)
        # Other models' boards are bare felt: no seasons, soot or sugar.
        sugarscape = d.model == "sugarscape"
        if sugarscape and d.config["seasons"]["enabled"]:
            updaters.append(board.seasonal_felt(felt, d, timing))
        elif sugarscape and any(any(f.pollution) for f in d.frames):
            updaters.append(board.sooty_felt(felt, d, timing))
        if sugarscape:
            _, update_sugar = board.sugar(d, corners, timing)
            updaters.append(update_sugar)
        if d.spice_capacity:
            _, update_spice = board.sugar(d, corners, timing, "spice")
            updaters.append(update_spice)
        # Only the Flumps alive during this beat's ticks get objects: a long
        # run with births can have many thousands over its whole length.
        first, last = timing.tick_at(1), timing.tick_at(beat.frames + 1)
        if d.model != "ethno":
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
