"""Physical coordinates shared by agreement panels and uncertainty markers."""
def interval(opinion, uncertainty):
    return opinion - uncertainty, opinion + uncertainty


def overlap(x1, u1, x2, u2):
    a, b = interval(x1, u1)
    c, d = interval(x2, u2)
    return min(b, d) - max(a, c)


def marker_length(uncertainty):
    return max(0.0, uncertainty) * 1.6


def diagram_point(period, opinion, last_period):
    return min(period / max(last_period, 1), 1), (opinion + 1) / 2


def uncertainty_color(uncertainty):
    t = min(max(uncertainty / 2, 0), 1)
    return tuple(a + (b - a) * t for a, b in zip((1.0, .2, .15), (.2, .9, .35)))


def select_pair(opinions, uncertainties):
    """A genuine initially confident agent and its most-overlapping moderate."""
    ids = list(opinions)
    extreme = min(ids, key=lambda i: uncertainties[i])
    candidates = [i for i in ids if i != extreme and uncertainties[i] > uncertainties[extreme]]
    other = max(candidates, key=lambda i: overlap(opinions[extreme], uncertainties[extreme], opinions[i], uncertainties[i]))
    return extreme, other


def outcome_clock(stats, horizon, stop_when_stable=True):
    """Only measured change establishes stability; stable_at may be a fallback."""
    def last(key, fallback):
        value = stats.get(key, fallback)
        return (value[-1] if value else fallback) if isinstance(value, (list, tuple)) else value
    stable_at = last('stable_at', 0)
    change = last('max_change', float('inf'))
    if change <= 1e-6 and stable_at > 0:
        return f'settled at {int(stable_at):,}'
    return f"{'capped' if stop_when_stable else 'stopped'} at {int(horizon):,}"
