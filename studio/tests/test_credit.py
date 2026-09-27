import unittest

import credit
from dump import Loan


def loan(lender, borrower):
    return Loan(lender, borrower, 10.0, 5)


class CreditTest(unittest.TestCase):
    def test_pure_lenders_are_level_one_and_each_borrower_sits_below_its_deepest_lender(self):
        loans = [loan(1, 2), loan(2, 3), loan(1, 3), loan(4, 3), loan(3, 5)]
        self.assertEqual(credit.levels(loans), {1: 1, 2: 2, 3: 3, 4: 1, 5: 4})
        self.assertEqual(credit.depth(loans), 4)
        self.assertEqual(credit.both(loans), {2, 3})

    def test_a_cycle_does_not_loop(self):
        levels = credit.levels([loan(1, 2), loan(2, 1)])
        self.assertEqual(set(levels), {1, 2})
        self.assertLessEqual(max(levels.values()), 2)

    def test_a_cycle_with_a_tail_counts_the_longest_path_round_it(self):
        # 1 → 2 → 3 → 1 is a cycle; 3 also lends to 4. The longest chain to 4
        # goes round the cycle once: 1, 2, 3, 4 (or 2, 3, 1 … not to 4).
        loans = [loan(1, 2), loan(2, 3), loan(3, 1), loan(3, 4)]
        self.assertEqual(credit.levels(loans)[4], 4)

    def test_levels_match_a_brute_force_longest_path_on_random_networks(self):
        import itertools
        import random
        rng = random.Random(7)
        for _ in range(50):
            loans = [loan(rng.randrange(7), rng.randrange(7)) for _ in range(9)]
            loans = [l for l in loans if l.lender != l.borrower]
            members = {x for l in loans for x in (l.lender, l.borrower)}
            edges = {(l.lender, l.borrower) for l in loans}
            best = {m: 1 for m in members}
            for k in range(2, len(members) + 1):
                for path in itertools.permutations(members, k):
                    if all((a, b) in edges for a, b in zip(path, path[1:])):
                        best[path[-1]] = max(best[path[-1]], k)
            self.assertEqual(credit.levels(loans), best)

    def test_no_loans_no_depth(self):
        self.assertEqual(credit.depth([]), 0)


if __name__ == "__main__":
    unittest.main()
