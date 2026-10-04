"""Pure recorded-agent appearance and polska motion for retirement only."""
import math
from dataclasses import dataclass, fields

BAR_SECONDS = 180 / 104

@dataclass(frozen=True)
class DancePose:
    offset: tuple = (0., 0., 0.)
    yaw: float = 0.
    lean: float = 0.
    left_foot: tuple = (0., 0., 0.)
    right_foot: tuple = (0., 0., 0.)
    arm_swing: float = 0.

@dataclass(frozen=True)
class AgeStyle:
    elderly: bool = False
    spectacles: bool = False
    stitches: int = 0


def dance_pose(seconds, member):
    seed = member['id'] * 17 + member['born'] * 31
    phase = 2 * math.pi * seconds / BAR_SECONDS
    if not member['retired']:
        return DancePose(lean=.025 * math.sin(phase / 8 + seed % 11))
    variation = .8 + (seed % 7) / 20
    step = math.sin(phase)
    lift_left = .075 * max(0., step) ** 2
    lift_right = .075 * max(0., -step) ** 2
    return DancePose(
        offset=(.135 * variation * math.sin(phase / 2), 0., .009 * (1-math.cos(phase * 3))),
        yaw=.12 * variation * math.sin(phase / 4 + (seed % 13) / 13),
        lean=.045 * variation * step,
        left_foot=(0., .075 * step, lift_left),
        right_foot=(0., -.075 * step, lift_right),
        arm_swing=.10 * variation * step,
    )


def age_style(member, hero=False):
    elderly = member['age'] >= 65
    seed = member['id'] * 17 + member['born'] * 31
    return AgeStyle(elderly, elderly and (hero or seed % 4 == 0),
                    (2 if member['age'] < 80 else 4) if elderly else 0)


def eligible_member(members, kind, eligibility):
    for slot in sorted(members):
        member = members[slot]
        if member['kind'] == kind and member['age'] >= eligibility:
            return member
    raise ValueError(f'no recorded {kind} member eligible at age {eligibility}')


def _ease(value):
    value = max(0., min(1., value))
    return value * value * (3 - 2 * value)


def join_amount(elapsed, onset):
    """Two-second presentation ramp, starting at the recorded switch clock."""
    return _ease((elapsed - onset) / 2)


def join_pose(seconds, member, amount):
    """Blend amplitude at the continuous musical phase; source stays untouched."""
    idle = dance_pose(seconds, dict(member, retired=False))
    if not member['retired']:
        return idle
    full = dance_pose(seconds, member)
    amount = max(0., min(1., amount))
    def blend(a, b):
        if isinstance(a, tuple):
            return tuple(blend(x, y) for x, y in zip(a, b))
        return a + (b-a)*amount
    return DancePose(**{f.name: blend(getattr(idle, f.name), getattr(full, f.name))
                        for f in fields(DancePose)})


def invitation_gaze(elapsed, targets, onset=None):
    """Look, hold, then scan actual counted friends; release while joining."""
    if not targets:
        return 0.
    gaze = targets[0] * _ease(elapsed / 1.2)
    for index, target in enumerate(targets[1:], 1):
        gaze += (target-targets[index-1]) * _ease((elapsed-2.4*index) / 1.2)
    if onset is not None:
        gaze *= 1 - join_amount(elapsed, onset)
    return gaze


def attention_angle(origin, target):
    """Bounded upper-body turn toward an actual stage position."""
    return max(-.28, min(.28, .5 * math.atan2(
        target[0]-origin[0], 4 + abs(target[1]-origin[1]))))
