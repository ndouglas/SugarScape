"""Checks that each episode's beats, overlays, measurements and tune agree,
without Blender: an overlay name nothing builds, a panel key the
measurements lack or a cue on a renamed beat would otherwise surface
mid-render, or (the cue) never."""

import ast
import json
import pathlib
import string
import unittest

import episode

OVERLAYS = pathlib.Path(__file__).resolve().parent.parent / "blender" / "overlays"
COLOR_MODES = {"family", "tribe", "sick", "strategy"}


def overlay_sources():
    return sorted(OVERLAYS.glob("*.py")) if OVERLAYS.is_dir() else [OVERLAYS.with_suffix(".py")]


def overlay_registry():
    """From the overlay modules' source: BUILDERS as {overlay name: function
    name}, SCREEN, and for each function the attributes it reads from
    `ctx`."""
    builders, screen, reads = {}, set(), {}
    for path in overlay_sources():
        tree = ast.parse(path.read_text(), str(path))
        for node in tree.body:
            if isinstance(node, ast.Assign) and isinstance(node.targets[0], ast.Name):
                target = node.targets[0].id
                if target == "BUILDERS":
                    builders = {k.value: v.id for k, v in zip(node.value.keys, node.value.values)}
                elif target == "SCREEN":
                    screen = {e.value for e in node.value.elts}
            elif isinstance(node, ast.FunctionDef):
                reads[node.name] = {
                    n.attr for n in ast.walk(node)
                    if isinstance(n, ast.Attribute) and isinstance(n.value, ast.Name) and n.value.id == "ctx"
                }
    return builders, screen, reads


def episodes():
    return sorted(p.name for p in (episode.ROOT / "episodes").iterdir() if (p / "beats.py").exists())


class WiringTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.builders, cls.screen, cls.reads = overlay_registry()

    def test_the_registry_is_found(self):
        self.assertIn("bars", self.builders)
        self.assertIn("compare", self.reads["counter"])
        self.assertIn("measured", self.reads["bars"])

    def test_every_builder_and_screen_overlay_exists(self):
        for name, function in self.builders.items():
            self.assertIn(function, self.reads, name)
        self.assertLessEqual(self.screen, set(self.builders))

    def test_every_overlay_a_beat_names_is_built_and_gets_its_data(self):
        for name in episodes():
            for b in episode.load_episode(name):
                for overlay in b.overlays:
                    with self.subTest(episode=name, beat=b.name, overlay=overlay):
                        self.assertIn(overlay, self.builders)
                        reads = self.reads[self.builders[overlay]]
                        if "compare" in reads:
                            self.assertTrue(b.compare, "reads a second shot, but the beat sets no compare")
                        if {"tracks", "timing", "corners"} & reads:
                            self.assertTrue(b.shot, "follows the Flumps, but the beat has no shot")

    def test_panels_only_show_measured_values(self):
        for name in episodes():
            path = episode.episode_dir(name) / "measurements.json"
            medians = json.loads(path.read_text())["medians"] if path.exists() else {}
            for b in episode.load_episode(name):
                # An overlay nothing builds is the test above's failure, not this one's.
                if not any("measured" in self.reads.get(self.builders.get(o), ()) for o in b.overlays):
                    continue
                with self.subTest(episode=name, beat=b.name):
                    self.assertTrue(path.exists(), "the panel needs measurements.json (run measure.py)")
                    rows = b.params.get("rows", [])
                    if "ledger" in b.overlays:
                        # (the book, here) rows; here's {key} placeholders name medians.
                        for book, here in rows:
                            for _, key, _, _ in string.Formatter().parse(here):
                                if key:
                                    self.assertIn(key, medians, book)
                        continue
                    for group in rows:
                        for label, key in group:
                            self.assertIn(key, medians, label)
                    if "tops" in b.params:
                        self.assertEqual(len(b.params["tops"]), len(rows))

    def test_beats_have_unique_names_and_known_color_modes(self):
        for name in episodes():
            beats = episode.load_episode(name)
            names = [b.name for b in beats]
            self.assertEqual(len(names), len(set(names)), name)
            for b in beats:
                self.assertIn(b.params.get("colors", "family"), COLOR_MODES, (name, b.name))

    def test_every_cue_names_a_beat_and_a_sting(self):
        # music.cue_times skips a cue whose beat is missing, so a renamed
        # beat would silently lose its sting.
        for name in episodes():
            tune = episode.load_module(name, "tune").TUNE
            names = {b.name for b in episode.load_episode(name)}
            for beat, sting, *_ in tune.cues:
                with self.subTest(episode=name, cue=(beat, sting)):
                    self.assertIn(beat, names)
                    self.assertIn(sting, tune.stings)
            self.assertLessEqual(set(tune.ducks), set(tune.stings), name)


if __name__ == "__main__":
    unittest.main()
