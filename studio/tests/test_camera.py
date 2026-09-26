import math
import unittest

import camera

A = ((0, -10, 5), (0, 0, 0))
B = ((0, -20, 15), (0, 0, 0))


class CameraTest(unittest.TestCase):
    def test_smootherstep_endpoints_and_monotonic(self):
        self.assertEqual((camera.smootherstep(0), camera.smootherstep(1)), (0, 1))
        values = [camera.smootherstep(i / 20) for i in range(21)]
        self.assertEqual(values, sorted(values))

    def test_moves_hold_before_between_and_after(self):
        moves = [camera.Move(1, 3, *A, *B, lens0=35, lens1=50)]
        self.assertEqual(camera.camera_at(moves, 0), (A[0], A[1], 35))
        self.assertEqual(camera.camera_at(moves, 5), (B[0], B[1], 50))
        eye, _, lens = camera.camera_at(moves, 2)
        self.assertAlmostEqual(eye[1], -15)
        self.assertAlmostEqual(lens, 42.5)

    def test_a_gap_between_moves_holds_the_first_moves_end(self):
        moves = [camera.Move(0, 1, *A, *B), camera.Move(2, 3, *B, *A)]
        self.assertEqual(camera.camera_at(moves, 1.5)[0], B[0])

    def test_orbit_swings_around_the_target(self):
        moves = [camera.Move(0, 1, *A, *A, orbit=math.pi / 2)]
        eye, target, _ = camera.camera_at(moves, 1)
        self.assertAlmostEqual(eye[0], 10, places=6)
        self.assertAlmostEqual(eye[1], 0, places=6)
        self.assertEqual(eye[2], 5)


if __name__ == "__main__":
    unittest.main()
