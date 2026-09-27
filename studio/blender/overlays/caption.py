"""Each beat's caption, rendered on its own."""

import bpy

from blender import materials

from .parts import CREAM, INK, box, text


def caption_scene(body, y, preview, title=False):
    """A scene holding only the caption — rounded cream text over a soft dark
    shadow — on a transparent film, rendered once per beat and laid over the
    beat by the cut (which fades it), so depth of field never blurs it. A
    title caption is larger, over a dark scrim that dims the whole frame."""
    scene = bpy.context.scene
    for obj in list(bpy.data.objects):
        bpy.data.objects.remove(obj)
    scene.render.engine = "BLENDER_EEVEE"
    scene.render.resolution_x, scene.render.resolution_y = (960, 540) if preview else (1920, 1080)
    scene.render.film_transparent = True
    scene.render.image_settings.file_format = "PNG"
    scene.render.image_settings.color_mode = "RGBA"
    scene.view_settings.view_transform = "Standard"
    data = bpy.data.cameras.new("caption-camera")
    data.type = "ORTHO"
    data.ortho_scale = 2.0  # x spans [-1, 1]
    cam = bpy.data.objects.new("caption-camera", data)
    cam.location = (0, 0, 10)
    scene.collection.objects.link(cam)
    scene.camera = cam
    anchor = bpy.data.objects.new("caption", None)
    anchor.location = (0, y * 9 / 16, 0)
    scene.collection.objects.link(anchor)
    size = 0.105 if title else 0.085
    if title:
        scrim = materials.fading("scrim", INK, 1.0)
        scrim.node_tree.nodes["Mix"].inputs[0].default_value = 0.62
        box("scrim", scrim, None, location=(0, 0, -1), scale=(2.4, 1.4, 0.01))
    text("caption-shadow", body, size, materials.fading("caption-ink", INK, 1.0), anchor, location=(0.005, -0.006, -0.01))
    text("caption", body, size, materials.fading("caption", CREAM, 1.0), anchor)
