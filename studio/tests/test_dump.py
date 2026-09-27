import json
import pathlib
import unittest

import dump

# Made by `sugarscape shot tests/fixtures/tiny.json`.
FIXTURE = pathlib.Path(__file__).parent / "fixtures" / "tiny.frames.json"


class DumpTest(unittest.TestCase):
    def setUp(self):
        self.d = dump.load(FIXTURE)

    def test_header(self):
        self.assertEqual((self.d.width, self.d.height, self.d.ticks), (8, 8, 6))
        self.assertEqual(len(self.d.frames), 7)
        self.assertEqual(len(self.d.capacity), 64)
        self.assertEqual(len(self.d.placed), 3)

    def test_frames_index_agents_by_id(self):
        first = self.d.placed[0]
        a = self.d.frames[0].agents[first]
        self.assertEqual((a.x, a.y, a.vision, a.metabolism, a.sugar), (4, 4, 3, 1, 5.0))

    def test_deaths_are_keyed_by_id(self):
        starving = self.d.placed[1]
        self.assertEqual(self.d.frames[1].deaths, {starving: "starvation"})

    def test_tracks_cover_life_and_record_death(self):
        t = dump.tracks(self.d)
        starving = t[self.d.placed[1]]
        self.assertEqual(
            (starving.first, len(starving.cells), starving.death, starving.cause),
            (0, 1, 1, "starvation"),
        )
        late = t[self.d.placed[2]]
        self.assertEqual(late.first, 2)
        self.assertEqual(late.cells[0], (7, 7))
        self.assertIsNone(late.death)
        self.assertEqual(len(late.cells), 5)

    def test_survival_is_the_share_of_a_starting_group_alive_at_the_end(self):
        # Three start (two placed at tick 0 plus none else); the starving one dies.
        self.assertEqual(dump.survival(self.d, lambda a: True), 0.5)
        self.assertEqual(dump.survival(self.d, lambda a: a.metabolism >= 4), 0.0)
        self.assertTrue(dump.survival(self.d, lambda a: a.vision > 99) != dump.survival(self.d, lambda a: a.vision > 99))

    def test_frames_carry_pollution_or_zeros_for_older_dumps(self):
        self.assertEqual(self.d.frames[3].pollution, [0.0] * 64)
        raw = json.loads(FIXTURE.read_text())
        for f in raw["frames"]:
            f.pop("pollution", None)
        self.assertEqual(dump.parse(json.dumps(raw)).frames[3].pollution, [0.0] * 64)

    def test_births_map_each_newcomer_to_its_sex_and_parents(self):
        first = self.d.placed[0]
        sex, parents = self.d.frames[0].births[first]
        self.assertIn(sex, ("female", "male"))
        self.assertIsNone(parents)  # placed, not born of parents
        self.assertEqual(set(self.d.frames[2].births), set(self.d.frames[2].born))
        raw = json.loads(FIXTURE.read_text())
        for f in raw["frames"]:
            f.pop("births", None)
        old = dump.parse(json.dumps(raw))
        self.assertEqual(old.frames[0].births[first], (None, None))

    def test_frames_map_each_agent_to_its_tags_and_group(self):
        f = self.d.frames[3]
        self.assertEqual(set(f.tags), set(f.agents))
        self.assertEqual(set(f.groups), set(f.agents))
        self.assertTrue(all(set(t) <= {"0", "1"} for t in f.tags.values()))
        raw = json.loads(FIXTURE.read_text())
        for fr in raw["frames"]:
            fr.pop("tags", None)
            fr.pop("groups", None)
        self.assertEqual(dump.parse(json.dumps(raw)).frames[3].groups, {})

    def test_one_good_dumps_have_no_spice_or_trades(self):
        self.assertEqual(self.d.spice_capacity, [])
        self.assertTrue(all(f.spice == [] and f.spice_agents == {} and f.trades == [] for f in self.d.frames))

    def test_wrong_format_is_refused(self):
        with self.assertRaisesRegex(ValueError, "format"):
            dump.parse('{"format": 99}')



# Made by `sugarscape shot tests/fixtures/market.json`: iv-3-trade on a
# 12 × 12 board, sugar in one corner and spice in the other.
MARKET = pathlib.Path(__file__).parent / "fixtures" / "market.frames.json"


class MarketDumpTest(unittest.TestCase):
    def setUp(self):
        self.d = dump.load(MARKET)

    def test_spice_is_recorded_per_site_and_per_agent(self):
        self.assertEqual(len(self.d.spice_capacity), 144)
        f = self.d.frames[2]
        self.assertEqual(len(f.spice), 144)
        self.assertEqual(set(f.spice_agents), set(f.agents))
        held, metabolism = f.spice_agents[next(iter(f.agents))]
        self.assertGreaterEqual(held, 0)
        self.assertGreater(metabolism, 0)

    def test_trades_name_both_partners_and_what_moved(self):
        self.assertEqual(self.d.frames[0].trades, [])
        t = self.d.frames[1].trades[0]
        self.assertIsInstance(t, dump.Trade)
        self.assertNotEqual(t.sugar_giver, t.spice_giver)
        self.assertGreater(t.sugar, 0)
        self.assertGreater(t.spice, 0)
        self.assertGreaterEqual(t.exchanges, 1)
        # Both partners are alive in the frame the trade is recorded in.
        self.assertIn(t.sugar_giver, self.d.frames[1].agents)
        self.assertIn(t.spice_giver, self.d.frames[1].agents)

    def test_a_trades_price_is_spice_per_sugar(self):
        t = self.d.frames[1].trades[0]
        self.assertAlmostEqual(t.price, t.spice / t.sugar)



# Made by `sugarscape shot tests/fixtures/war.json`: a rich Blue among two
# poor Reds on an 8 × 8 board with combat on.
WAR = pathlib.Path(__file__).parent / "fixtures" / "war.frames.json"


class WarDumpTest(unittest.TestCase):
    def test_kills_name_attacker_victim_and_loot_and_match_combat_deaths(self):
        d = dump.load(WAR)
        blue = d.placed[0]
        kills = [k for f in d.frames for k in f.kills]
        self.assertTrue(kills)
        for f in d.frames:
            self.assertEqual({k.victim for k in f.kills}, {i for i, c in f.deaths.items() if c == "combat"})
        self.assertTrue(all(isinstance(k, dump.Kill) and k.attacker == blue and k.loot > 0 for k in kills))

    def test_dumps_without_combat_have_no_kills(self):
        self.assertTrue(all(f.kills == [] for f in dump.load(FIXTURE).frames))


if __name__ == "__main__":
    unittest.main()
