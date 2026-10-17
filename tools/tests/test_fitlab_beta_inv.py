"""The inverse beta fit (T-1745): the bisection solver and the scores the docs quote."""

from __future__ import annotations

import pytest

from rustsheet_tools.fitlab import fit
from rustsheet_tools.fitlab.kernels import beta, beta_inv, registry


def test_bisection_lands_on_adjacent_doubles() -> None:
    # I_x(1,1) = x: the crossing of p = 0.3 is the double 0.3 itself.
    assert beta_inv.bisect(beta.ibeta_reference, 0.3, 1.0, 1.0, "hi") == 0.3
    lo = beta_inv.bisect(beta.ibeta_reference, 0.3, 1.0, 1.0, "lo")
    assert lo < 0.3 and beta.ibeta_reference(lo, 1.0, 1.0) < 0.3
    with pytest.raises(ValueError, match="outside"):
        beta_inv.bisect(beta.ibeta_reference, 1.0, 2.0, 3.0, "mid")


@pytest.mark.usefixtures("real_cases")
def test_bisection_on_cephes_score() -> None:
    # The correctly rounded rows (87 for the midpoint) take minutes at 70 digits; Cephes is quick.
    result = fit.run(registry()["beta-inv"], engine=False, only=r"\(mid\) on Cephes")
    assert len(result.sweep.cells) == 212
    assert [s.exact for s in result.scores] == [73]


def test_f_and_t_inverses() -> None:
    cephes = beta_inv.FORWARD["Cephes incbet as published"]
    # F(2,2) has CDF x/(1+x): the median is 1. The t distribution with 1 df is Cauchy.
    assert beta_inv.f_inv(cephes, 0.5, 2.0, 2.0, "hi", right_tail=False) == pytest.approx(1.0, rel=1e-15)
    assert beta_inv.f_inv(cephes, 0.5, 2.0, 2.0, "hi", right_tail=True) == pytest.approx(1.0, rel=1e-15)
    assert beta_inv.t_inv(cephes, 0.75, 1.0, "mid", two_tailed=False) == pytest.approx(1.0, rel=1e-14)
    assert beta_inv.t_inv(cephes, 0.25, 1.0, "mid", two_tailed=False) == pytest.approx(-1.0, rel=1e-14)
    assert beta_inv.t_inv(cephes, 0.5, 1.0, "mid", two_tailed=True) == pytest.approx(1.0, rel=1e-14)
    assert beta_inv.t_inv(cephes, 0.5, 7.0, "mid", two_tailed=False) == 0.0
    with pytest.raises(ValueError, match="end must be"):
        beta_inv.crossing(lambda x: x, 0.5, 0.0, 1.0, "high")
