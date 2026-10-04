"""Whole-history predictive draws and independent treatment contrasts."""
import bisect
import math
import numpy as np


def inclusive_p(sorted_draws, target):
    if not sorted_draws:
        raise ValueError('empty predictive distribution')
    n = len(sorted_draws)
    left = bisect.bisect_right(sorted_draws, target)
    right = n - bisect.bisect_left(sorted_draws, target)
    return min(1., 2 * (1 + min(left, right)) / (n + 1))


def maximize_interval_p(sorted_draws, interval):
    lo, hi = interval
    if not all(math.isfinite(x) for x in (lo, hi)) or lo > hi:
        raise ValueError('invalid source interval')
    if not sorted_draws or any(not math.isfinite(x) for x in sorted_draws):
        raise ValueError('invalid predictive distribution')
    if any(a > b for a, b in zip(sorted_draws, sorted_draws[1:])):
        raise ValueError('predictive distribution must be sorted')
    a = bisect.bisect_left(sorted_draws, lo)
    b = bisect.bisect_right(sorted_draws, hi)
    knots = sorted(set([lo, hi] + sorted_draws[a:b]))
    candidates = [(inclusive_p(sorted_draws, x), 'point', x) for x in knots]
    n = len(sorted_draws)
    # No sample lies strictly between consecutive knots. Compute the open-gap
    # tails directly, even when adjacent float endpoints have no float midpoint.
    for left, right in zip(knots, knots[1:]):
        below = bisect.bisect_right(sorted_draws, left)
        above = n - bisect.bisect_left(sorted_draws, right)
        p = min(1., 2 * (1 + min(below, above)) / (n + 1))
        candidates.append((p, 'open_interval', [left, right]))
    maximum = max(candidate[0] for candidate in candidates)
    winner = next(candidate for candidate in reversed(candidates) if candidate[0] == maximum)
    return {'p': maximum, 'maximizer_kind': winner[1], 'maximizer': winner[2],
            'tested_point_count': len(knots), 'tested_open_interval_count': max(0, len(knots) - 1)}


def fixed_holm(values, family_size):
    if len(values) != family_size:
        raise ValueError('fixed hypothesis family size mismatch')
    internal = [1. if value is None else value for value in values]
    if any(not math.isfinite(p) or not 0 <= p <= 1 for p in internal):
        raise ValueError('invalid p-value')
    adjusted = [None] * family_size
    running = 0.
    for rank, i in enumerate(sorted(range(family_size), key=lambda i: (internal[i], i))):
        running = min(1., max(running, (family_size - rank) * internal[i]))
        if values[i] is not None:
            adjusted[i] = running
    return adjusted


def percentile_interval(values):
    if not values:
        raise ValueError('empty interval sample')
    ordered = sorted(values)
    return [ordered[math.floor(.025 * (len(ordered) - 1))],
            ordered[math.ceil(.975 * (len(ordered) - 1))]]


def predictive(values, rng, *, draws=100000, minimum_defined=10000):
    if len(values) != 100 or type(draws) is not int or draws <= 0:
        raise ValueError('one hundred histories and positive draw count required')
    if type(minimum_defined) is not int or minimum_defined <= 0:
        raise ValueError('positive conditional sufficiency count required')
    mask = np.asarray([x is not None for x in values], dtype=bool)
    finite = np.asarray([0. if x is None else x for x in values], dtype=float)
    if not np.all(np.isfinite(finite)):
        raise ValueError('nonfinite history statistic')
    conditional = not np.all(mask)
    result = {'status': 'Unavailable', 'reason': None, 'requested_draws': draws,
        'attempted_draws': 0, 'defined_draws': 0, 'undefined_draws': 0,
        'surviving_precision_histories': int(mask.sum()), 'replicates': [],
        'predictive_interval': None, 'conditional': conditional}
    blocks = []
    for start in range(0, draws, 512):
        indices = rng.integers(0, 100, size=(min(512, draws - start), 30), dtype=np.int64)
        counts = mask[indices].sum(axis=1)
        sums = finite[indices].sum(axis=1)
        defined = counts > 0
        blocks.append((sums[defined] / counts[defined]).tolist())
    samples = sorted(x for block in blocks for x in block)
    result.update(attempted_draws=draws, defined_draws=len(samples),
        undefined_draws=draws - len(samples),
        undefined_fraction=(draws - len(samples)) / draws, replicates=samples)
    if conditional and (mask.sum() < 2 or len(samples) < minimum_defined):
        result['reason'] = 'insufficient_conditional_reference'
        return result
    result.update(status='Available', predictive_interval=percentile_interval(samples))
    return result


def contrast(left, right, permutation_rng, bootstrap_rng, *, draws=100000):
    if len(left) != 100 or len(right) != 100 or type(draws) is not int or draws <= 0:
        raise ValueError('two complete independent hundred-history groups required')
    left = np.asarray(left, dtype=float)
    right = np.asarray(right, dtype=float)
    if not np.all(np.isfinite(left)) or not np.all(np.isfinite(right)):
        raise ValueError('nonfinite contrast statistic')
    observed = float(left.mean() - right.mean())
    pooled = np.concatenate((left, right))
    exceedances = 0
    for _ in range(draws):
        shuffled = permutation_rng.permutation(pooled)
        difference = float(shuffled[:100].mean() - shuffled[100:].mean())
        exceedances += abs(difference) >= abs(observed)
    bootstrap = []
    for start in range(0, draws, 256):
        count = min(256, draws - start)
        il = bootstrap_rng.integers(0, 100, size=(count, 100), dtype=np.int64)
        ir = bootstrap_rng.integers(0, 100, size=(count, 100), dtype=np.int64)
        bootstrap.extend((left[il].mean(axis=1) - right[ir].mean(axis=1)).tolist())
    return {'estimate': observed, 'p': (1 + int(exceedances)) / (draws + 1),
        'permutation_exceedances': int(exceedances), 'permutations': draws,
        'bootstrap_draws': draws, 'interval': percentile_interval(bootstrap),
        'resampling': 'independent_whole_history'}
