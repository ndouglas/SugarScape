"""Credit: the loan network, its levels, and who lends to whom (see
episodes/credit)."""

import collections


def levels(loans):
    """Each Flump in the loan network's level: 1 for pure lenders, and one
    more than its deepest lender for everyone else (a lender's lender is
    above it). Cycles, which the rule allows, are broken where first met."""
    lenders_of = collections.defaultdict(set)
    members = set()
    for l in loans:
        lenders_of[l.borrower].add(l.lender)
        members |= {l.lender, l.borrower}
    memo = {}

    def level(a, path):
        if a in memo:
            return memo[a]
        above = [level(b, path | {a}) for b in lenders_of.get(a, ()) if b not in path]
        memo[a] = 1 + max(above, default=0)
        return memo[a]

    return {a: level(a, frozenset()) for a in sorted(members)}


def depth(loans):
    """The loan network's deepest level: the longest chain of debt, in Flumps."""
    return max(levels(loans).values(), default=0)


def both(loans):
    """The Flumps that are lenders and borrowers at once."""
    return {l.lender for l in loans} & {l.borrower for l in loans}
