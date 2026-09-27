import unittest

import smoke
from episode import Beat


class SmokeTest(unittest.TestCase):
    def test_takes_the_first_beat_of_each_kind(self):
        beats = [
            Beat("a", "", 1, shot="s", overlays=("bars",)),
            Beat("b", "", 1, shot="t", overlays=("bars",)),  # same kind as a
            Beat("c", "", 1, shot="s", overlays=("bars",), closeup=True),
            Beat("d", "", 1, shot="s", overlays=("alike", "bars")),
            Beat("e", "", 1, shot="s", overlays=("bars", "alike")),  # same kind as d
            Beat("f", "", 1, shot="s", params={"colors": "tribe"}),
            Beat("g", "", 1),  # the title card
            Beat("h", "", 1, title=True),
        ]
        self.assertEqual(smoke.representatives(beats), [1, 3, 4, 6, 7, 8])


if __name__ == "__main__":
    unittest.main()
