"""Exact discrete-model checks; no empirical or independent-validation claim."""
from fractions import Fraction as F
from itertools import product
import unittest


def midpoint(x, v, h, k, d):
    denominator = 4 + 2 * h * d + h * h * k
    return (((4 + 2 * h * d - h * h * k) * x + 4 * h * v) / denominator,
            (-4 * h * k * x + (4 - 2 * h * d - h * h * k) * v) / denominator)


class OscillatorModelTests(unittest.TestCase):
    def test_exact_balance_and_dissipation_over_trajectories(self):
        for x, v, h, k, d in product(map(F, [-1, 0, 1]), map(F, [-1, 0, 1]),
                                   [F(0), F(1, 10), F(1)],
                                   [F(0), F(1), F(3)], [F(0), F(1, 2), F(2)]):
            for _ in range(10):
                y, w = midpoint(x, v, h, k, d)
                self.assertEqual(2 * (y - x), h * (v + w))
                self.assertEqual(2 * (w - v), -h * (k * (x + y) + d * (v + w)))
                delta = k * y * y + w * w - (k * x * x + v * v)
                self.assertEqual(2 * delta, -h * d * (v + w) ** 2)
                self.assertLessEqual(delta, 0)
                if d == 0:
                    self.assertEqual(delta, 0)
                x, v = y, w

    def test_undamped_step_is_reversible(self):
        for x, v, h, k in product(map(F, [-2, 0, 3]), map(F, [-1, 0, 1]),
                                [F(1, 10), F(1)], [F(0), F(1), F(3)]):
            y, w = midpoint(x, v, h, k, F(0))
            self.assertEqual(midpoint(y, w, -h, k, F(0)), (x, v))

    def test_forward_euler_counterexample(self):
        x, v, h = F(1), F(0), F(1)
        y, w = x + h * v, v - h * x
        self.assertEqual(y * y + w * w, 2 * (x * x + v * v))
        self.assertNotEqual(y * y + w * w, x * x + v * v)


if __name__ == "__main__":
    unittest.main()
