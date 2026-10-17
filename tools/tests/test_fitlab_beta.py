"""The incomplete beta fits (T-1744): the reference, Excel's closed forms, and the scores the docs quote."""

from __future__ import annotations

from fractions import Fraction

import pytest

from rustsheet_tools.fitlab import fit
from rustsheet_tools.fitlab.kernels import beta, registry


def test_reference_and_cephes_agree_on_closed_forms() -> None:
    # I_x(2,2) = 3x^2 - 2x^3 at the double nearest 0.1; I_x(a,1) = x^a.
    x = Fraction(0.1)
    assert beta.ibeta_reference(0.1, 2.0, 2.0) == float(3 * x**2 - 2 * x**3)
    assert beta.ibeta_reference(0.5, 50.0, 1.0) == 0.5**50
    assert beta.cephes_incbet(0.1, 2.0, 2.0, beta.libm_pow, beta.gamma_ratio_libm) == pytest.approx(
        0.028, rel=1e-15
    )
    assert beta.nr_betai(0.1, 2.0, 2.0) == pytest.approx(0.028, rel=1e-14)


def test_excel_closed_forms() -> None:
    # Excel: BETA.DIST(0.1,5,1,TRUE) = exp(5 ln 0.1) = 1.0000000000000016e-05 (m9-beta-dist-a5), not 1e-5.
    assert beta.closed_form(0.1, 5.0, 1.0) == 1.0000000000000016e-05
    # BETA.DIST(0.1,1,2,TRUE) = 1 - 0.9^2 in doubles (m9-beta-dist-a1).
    assert beta.closed_form(0.1, 1.0, 2.0) == 0.18999999999999995
    assert beta.closed_form(0.1, 2.0, 2.0) is None


@pytest.mark.usefixtures("real_cases")
def test_best_combination_beats_the_reference_on_beta_dist() -> None:
    result = fit.run(registry()["beta-dist"], engine=False, only=r"^(correctly rounded|closed forms)")
    scores = {s.name: s.exact for s in result.scores}
    assert len(result.sweep.cells) == 283
    assert scores["closed forms, Cephes fractions, mean switch (best)"] == 158
    assert scores["correctly rounded"] == 130
