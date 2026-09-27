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

    def test_no_loans_no_depth(self):
        self.assertEqual(credit.depth([]), 0)


if __name__ == "__main__":
    unittest.main()
