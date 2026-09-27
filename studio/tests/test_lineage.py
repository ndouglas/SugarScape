import unittest

import dump
import lineage


def frame(tick, births):
    return dump.Frame(tick, {}, [], {}, list(births), [], dict(births))


class FakeDump:
    """Founders 1 (female) and 2 (male); their children 3 and 4 at tick 5;
    3 (female) and a newcomer 5 (male, placed at tick 5) have 6 at tick 9."""

    frames = [
        frame(0, {1: ("female", None), 2: ("male", None)}),
        frame(5, {3: ("female", (2, 1)), 4: ("male", (1, 2)), 5: ("male", None)}),
        frame(9, {6: ("male", (5, 3))}),
    ]


class LineageTest(unittest.TestCase):
    d = FakeDump()

    def test_generations_count_from_the_founders(self):
        self.assertEqual(lineage.generations(self.d), {1: 0, 2: 0, 3: 1, 4: 1, 5: 0, 6: 2})

    def test_families_follow_the_mothers_line(self):
        self.assertEqual(lineage.families(self.d), {1: 1, 2: 2, 3: 1, 4: 1, 5: 5, 6: 1})

    def test_children_and_birth_ticks(self):
        self.assertEqual(lineage.children(self.d), {1: [3, 4], 2: [3, 4], 3: [6], 5: [6]})
        self.assertEqual(lineage.born_at(self.d)[6], 9)

    def test_bequests_go_to_the_children_alive_when_a_parent_dies(self):
        a = lambda i, sugar: dump.Agent(i, 0, 0, sugar, 0, 1, 1)  # noqa: E731
        frames = [
            dump.Frame(0, {1: a(1, 5), 3: a(3, 1), 4: a(4, 1)}, [], {}, [1, 3, 4], [],
                       {1: ("female", None), 3: ("female", (1, 2)), 4: ("male", (1, 2))}),
            dump.Frame(1, {3: a(3, 7), 4: a(4, 1)}, [], {1: "old_age"}, [], [], {}),
            dump.Frame(2, {3: a(3, 7)}, [], {4: "starvation"}, [], [], {}),
        ]

        class D:
            pass

        d = D()
        d.frames, d.config = frames, {"inheritance": {"enabled": True}}
        self.assertEqual(lineage.bequests(d), [(1, 1, 5, [3, 4])])  # 4 died broke: nothing to leave
        d.config = {"inheritance": {"enabled": False}}
        self.assertEqual(lineage.bequests(d), [])

    def test_spearman_ranks(self):
        self.assertAlmostEqual(lineage.spearman([1, 2, 3, 4], [10, 20, 30, 40]), 1.0)
        self.assertAlmostEqual(lineage.spearman([1, 2, 3, 4], [4, 3, 2, 1]), -1.0)


if __name__ == "__main__":
    unittest.main()
