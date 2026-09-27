"""Credit: the loan network, its levels, and who lends to whom (see
episodes/credit)."""

import collections


def levels(loans):
    """Each Flump in the loan network's level: the number of Flumps on the
    longest chain of lenders ending at it that visits no Flump twice (1 for a
    pure lender). Cycles, which the rule allows, are followed only once
    round. Exact: the networks are small and sparse."""
    lenders_of = collections.defaultdict(set)
    members = set()
    for l in loans:
        lenders_of[l.borrower].add(l.lender)
        members |= {l.lender, l.borrower}

    def longest(a, path):
        return 1 + max((longest(b, path | {b}) for b in lenders_of.get(a, ()) if b not in path), default=0)

    return {a: longest(a, frozenset({a})) for a in sorted(members)}


def depth(loans):
    """The loan network's deepest level: the longest chain of debt, in Flumps."""
    return max(levels(loans).values(), default=0)


def both(loans):
    """The Flumps that are lenders and borrowers at once."""
    return {l.lender for l in loans} & {l.borrower for l in loans}
