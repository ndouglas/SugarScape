import pathlib
import tempfile
import unittest

import cut


def graph(argv):
    return argv[argv.index("-filter_complex") + 1]


class CutTest(unittest.TestCase):
    def test_total_frames_subtracts_dissolves(self):
        self.assertEqual(cut.total_frames([90, 60, 30], 12), 180 - 24)
        self.assertEqual(cut.total_frames([90], 12), 90)

    def test_one_beat_without_a_caption_is_a_plain_encode(self):
        argv = cut.command(["b/01"], [90], "o.mp4")
        self.assertIn("b/01/%04d.png", argv)
        self.assertNotIn("-filter_complex", argv)
        self.assertEqual(argv[-1], "o.mp4")
        self.assertIn("yuv420p", argv)

    def test_dissolve_offsets_accumulate(self):
        argv = cut.command(["a", "b", "c"], [90, 60, 30], "o.mp4", dissolve=12)
        g = graph(argv)
        # offsets: first at (90 − 12)/30 = 2.6 s; second at (90 + 60 − 24)/30 = 4.2 s
        self.assertIn("xfade=transition=fade:duration=0.4:offset=2.6", g)
        self.assertIn("xfade=transition=fade:duration=0.4:offset=4.2", g)
        self.assertEqual(argv[argv.index("-map") + 1], "[v2]")

    def test_captions_are_looped_faded_and_laid_over_their_beat(self):
        argv = cut.command(["a", "b"], [90, 60], "o.mp4", captions=["a/caption.png", None])
        # The caption is an extra input, looped for its beat's 3 s.
        i = argv.index("a/caption.png")
        self.assertEqual(argv[i - 7 : i], ["-loop", "1", "-framerate", "30", "-t", "3", "-i"])
        g = graph(argv)
        self.assertIn("[2:v]format=rgba,fade=t=in:st=0.3:d=0.4:alpha=1,fade=t=out:st=2.5:d=0.4:alpha=1[c0]", g)
        self.assertIn("[0:v][c0]overlay=shortest=1[b0]", g)
        self.assertIn("[b0][1:v]xfade", g)

    def test_a_single_captioned_beat_maps_its_overlay(self):
        argv = cut.command(["a"], [90], "o.mp4", captions=["a/caption.png"])
        self.assertEqual(argv[argv.index("-map") + 1], "[b0]")


class FolderCheckTest(unittest.TestCase):
    def folder(self, root, name, count, caption=False):
        f = root / name
        f.mkdir()
        for i in range(1, count + 1):
            (f / f"{i:04d}.png").write_bytes(b"")
        if caption:
            (f / "caption.png").write_bytes(b"")
        return str(f)

    def test_matching_folders_pass(self):
        with tempfile.TemporaryDirectory() as t:
            root = pathlib.Path(t)
            a = self.folder(root, "01", 3, caption=True)
            self.assertEqual(cut.check_folders([a], [3], [f"{a}/caption.png"]), [])

    def test_short_long_and_uncaptioned_beats_are_named(self):
        with tempfile.TemporaryDirectory() as t:
            root = pathlib.Path(t)
            short, long_, bare = (self.folder(root, "01", 2), self.folder(root, "02", 4), self.folder(root, "03", 3))
            problems = cut.check_folders([short, long_, bare], [3, 3, 3], [None, None, f"{bare}/caption.png"])
            self.assertEqual(len(problems), 3)
            self.assertIn("01", problems[0])
            self.assertIn("02", problems[1])
            self.assertIn("caption", problems[2])


if __name__ == "__main__":
    unittest.main()
