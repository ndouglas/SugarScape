"""Camera moves as pure functions of time: a beat's camera is a list of moves,
each easing eye, target and lens from a start pose to an end pose, with an
optional orbit around the target. Times are seconds from the beat's start."""

import math
from dataclasses import dataclass


def smootherstep(u):
    u = min(max(u, 0.0), 1.0)
    return u * u * u * (u * (u * 6 - 15) + 10)


@dataclass(frozen=True)
class Move:
    start: float
    end: float
    eye0: tuple
    target0: tuple
    eye1: tuple
    target1: tuple
    lens0: float = 50.0
    lens1: float = 50.0
    orbit: float = 0.0


def _lerp(a, b, t):
    return tuple(x + (y - x) * t for x, y in zip(a, b))


def _at(m, u):
    e = smootherstep(u)
    eye, target = _lerp(m.eye0, m.eye1, e), _lerp(m.target0, m.target1, e)
    if m.orbit:
        angle = m.orbit * e
        dx, dy = eye[0] - target[0], eye[1] - target[1]
        c, s = math.cos(angle), math.sin(angle)
        eye = (target[0] + dx * c - dy * s, target[1] + dx * s + dy * c, eye[2])
    return eye, target, m.lens0 + (m.lens1 - m.lens0) * e


def camera_at(moves, seconds):
    """Eye, target and lens: before the first move its start; between moves
    the previous move's end; after the last its end."""
    current = moves[0]
    if seconds <= current.start:
        return _at(current, 0.0)
    for m in moves:
        if seconds < m.start:
            return _at(current, 1.0)
        current = m
        if seconds <= m.end:
            return _at(m, (seconds - m.start) / max(m.end - m.start, 1e-9))
    return _at(current, 1.0)
