"""The incomplete beta fits (T-1744): the reference, Excel's closed forms, and the scores the docs quote."""

from __future__ import annotations

from fractions import Fraction

import pytest

from rustsheet_tools.fitlab import fit
from rustsheet_tools.fitlab.kernels import beta, registry
from rustsheet_tools.fitlab.score import ulp_distance
from rustsheet_tools.fitlab.sweep import Cell


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


def test_gamma_and_mappings() -> None:
    assert beta.gamma_cr(5.0) == 24.0
    assert beta.gamma_ratio_cr(2.0, 3.0) == 12.0
    t_dist = beta.t_dist_candidates()[0].evaluate
    f_dist = beta.f_dist_candidates()[0].evaluate

    def cell(*args: float) -> Cell:
        return Cell("case", "Calc", "A1", "", args, 0.0)

    assert t_dist(cell(0.0, 5.0)) == 0.5
    # Excel truncates the F df: F.DIST(1,3.9,7.9,TRUE) = F.DIST(1,3,7,TRUE) (t1049-fdist).
    assert f_dist(cell(1.0, 3.9, 7.9)) == f_dist(cell(1.0, 3.0, 7.0))


def test_libreoffice_beta_dist() -> None:
    # LibreOffice's own special cases versus Excel's.
    assert beta.libreoffice_beta_dist(0.1, 5.0, 1.0) == 0.1**5
    assert beta.libreoffice_beta_dist(0.1, 5.0, 1.0, excel_closed_forms=True) == 1.0000000000000016e-05
    assert beta.libreoffice_beta_dist(0.3, 2.5, 3.5) == pytest.approx(
        beta.ibeta_reference(0.3, 2.5, 3.5), rel=1e-14
    )


@pytest.mark.usefixtures("real_cases")
def test_libreoffice_scores() -> None:
    result = fit.run(registry()["beta-dist"], engine=False, only="^LibreOffice")
    assert {s.name: s.exact for s in result.scores} == {
        "LibreOffice GetBetaDist": 134,
        "LibreOffice GetBetaDist + Excel closed forms": 156,
    }


def test_t_cdf_reference_keeps_y_exact_in_the_deep_lower_tail() -> None:
    # T-2088: probes-numeric m9-t-dist-cdf A149/A150; rounding y first was 152,191 / 43,561 ulp off.
    assert ulp_distance(beta.t_cdf_reference(-2, 1e6), 0.022750266925659606) <= 1
    assert ulp_distance(beta.t_cdf_reference(-10, 1e6), 7.63930538408907e-24) == 38
    assert beta.t_cdf_reference(0, 5) == 0.5
    assert ulp_distance(beta.t_cdf_reference(2, 1e6), 1 - 0.022750266925659606) <= 1
