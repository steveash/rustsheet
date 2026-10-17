"""The inverse normal fit (T-1746): nested p lookup, the AS 241 tails, and the scores the docs quote."""

from __future__ import annotations

import pytest

from rustsheet_tools.fitlab import fit
from rustsheet_tools.fitlab.kernels import norm_inv, registry


def test_ppnd16_reproduces_published_quantiles() -> None:
    # Wichura's PPND16 is accurate to about 1e-16; these are the textbook quantiles.
    assert norm_inv.as241(0.975, norm_inv.PPND16, norm_inv.horner, log1p_upper=False) == pytest.approx(
        1.959963984540054, abs=1e-15
    )
    assert norm_inv.as241(0.5, norm_inv.PPND16, norm_inv.horner, log1p_upper=False) == 0.0
    # The double 0.975 is 2.2e-17 low, which moves the quantile 3.8e-16 below 1.95996398454005424.
    assert norm_inv.correctly_rounded(0.975) == 1.9599639845400538
    with pytest.raises(ValueError, match="outside"):
        norm_inv.as241(1.0, norm_inv.PPND16, norm_inv.horner, log1p_upper=False)


def test_statrs_port_matches_the_hybrid_centre() -> None:
    assert norm_inv.hybrid(0.3) == norm_inv.statrs_central(0.3)
    assert norm_inv.hybrid(0.01) == norm_inv.as241(0.01, norm_inv.PPND16, norm_inv.horner, log1p_upper=False)


@pytest.mark.usefixtures("real_cases")
def test_nested_p_is_excels_own_cdf() -> None:
    # NORM.S.DIST(-1,TRUE) is 2 ulp below the correctly rounded Phi(-1) in Excel.
    assert norm_inv.p_of((-1.0, True)) == 0.158655253931457
    assert norm_inv.p_of((0.3,)) == 0.3


@pytest.mark.usefixtures("real_cases")
def test_tails_are_ppnd16_and_the_hybrid_is_best() -> None:
    result = fit.run(registry()["norm-s-inv"], engine=False, only=r"^PPND16 (horner, upper log\(1-p\)|tails)")
    scores = {s.name: s.exact for s in result.scores}
    assert len(result.sweep.cells) == 1024
    assert scores["PPND16 tails, statrs centre"] == 775
    assert scores["PPND16 horner, upper log(1-p)"] == 693
    tails = [c for c in result.sweep.cells if abs(norm_inv.p_of(c.args) - 0.5) > norm_inv.SPLIT1]
    exact = [
        c
        for c in tails
        if norm_inv.as241(norm_inv.p_of(c.args), norm_inv.PPND16, norm_inv.horner, log1p_upper=False)
        == c.expected
    ]
    assert (len(exact), len(tails)) == (611, 616)


def test_other_candidates_are_close_to_ppnd16() -> None:
    reference = norm_inv.as241(0.975, norm_inv.PPND16, norm_inv.horner, log1p_upper=False)
    assert norm_inv.acklam(0.975, halley=True) == pytest.approx(reference, abs=1e-15)
    assert norm_inv.acklam(0.975, halley=False) == pytest.approx(reference, rel=2e-9)
    assert norm_inv.moro(0.975) == pytest.approx(reference, rel=1e-8)
    assert norm_inv.refine(0.975, reference, halley=False) == pytest.approx(reference, abs=1e-15)


@pytest.mark.usefixtures("real_cases")
def test_input_quirk_cells_are_left_out() -> None:
    result = fit.run(registry()["norm-s-inv"], engine=False, only="^$")
    keys = {f"{c.case} {c.key}" for c in result.sweep.cells}
    assert not keys & norm_inv._INPUT_QUIRKS
    assert result.sweep.excluded == 7  # the 3 quirk cells and 4 text-p cells


def test_cody_cdf_is_accurate() -> None:
    assert norm_inv.cody_cdf(0.0) == 0.5
    for x in (-37.0, -6.0, -1.5, 0.3, 2.0, 7.0):
        assert norm_inv.cody_cdf(x) == pytest.approx(norm_inv.libm_cdf(x), rel=1e-13)


@pytest.mark.usefixtures("real_cases")
def test_norm_s_dist_scores() -> None:
    result = fit.run(registry()["norm-s-dist"], engine=False)
    scores = {s.name: s.exact for s in result.scores}
    assert len(result.sweep.cells) == 1779
    assert scores["0.5*erfc(-x*sqrt(0.5)) (glibc)"] == 1103
    assert scores["Cody (R pnorm_both)"] == 991
