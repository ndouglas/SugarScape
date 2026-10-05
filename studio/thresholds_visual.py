"""Display planning from recorded threshold states; no invented events."""
import math


def join_hop(before, after, fraction):
    if not before and after and 0 < fraction < 1:
        return .45 * math.sin(math.pi * fraction)
    return 0.


def links(members):
    return sorted({tuple(sorted((i,j))) for i,a in members.items() for j in (a['neighbors'] or []) if i!=j})


def reached(member):
    if member['seed']: return True
    return member['of'] > 0 and member['threshold_num'] * member['of'] <= member['sees'] * member['threshold_den']


def people_counts(config):
    """A whole-crowd integer count is invalid for weighted friendship perception."""
    return config['network']=='everyone' and not config['friends']['enabled']
