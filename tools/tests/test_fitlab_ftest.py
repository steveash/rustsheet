"""The F.TEST fit's building blocks (T-2571)."""

from __future__ import annotations

import math

from rustsheet_tools.fitlab.kernels.ftest import beta_recurrence, one_pass_variance, two_pass_variance


def test_variances() -> None:
    assert two_pass_variance([1.0, 2.0, 3.0, 4.0]) == 5.0 / 3.0
    assert one_pass_variance([1.0, 2.0, 3.0, 4.0]) == 5.0 / 3.0


def test_beta_recurrence() -> None:
    # B(1/2,1/2) = pi, B(1,1) = 1, B(2,3) = 1/12, B(3/2,1/2) = pi/2.
    assert beta_recurrence(0.5, 0.5) == math.pi
    assert beta_recurrence(1.0, 1.0) == 1.0
    assert math.isclose(beta_recurrence(2.0, 3.0), 1.0 / 12.0, rel_tol=1e-15)
    assert math.isclose(beta_recurrence(1.5, 0.5), math.pi / 2.0, rel_tol=1e-15)
