"""Rule R of fitlab/kernels/gamma_huge_ratio.py and its Gate A aimed draw (T-4635)."""

from __future__ import annotations

import random
import sys

from rustsheet_tools.accuracy.defs import gamma_dist
from rustsheet_tools.fitlab.kernels import gamma_huge_ratio, registry


def test_rule_r_cuts() -> None:
    # The cells T-4877 names for gamma.rs's unit test (probes/t4635-gamma-dist-huge-ratio-cut and -fresh).
    rule = gamma_huge_ratio.rule_r
    assert not rule(1.5, 5e307)
    assert rule(1.5, 7e307)
    assert not rule(1.1, 1e280)
    assert rule(1.1, 1e282)
    assert not rule(1.11, 1e306)
    assert not rule(1.0, 1.79e308)


def test_aimed_cut_matches_rule_r() -> None:
    for a in (1.01, 1.05, 1.1, 1.11, 1.5, 2.0, 3.9):
        cut = gamma_dist._huge_ratio_cut(a)
        assert gamma_huge_ratio.rule_r(a, cut * 1.001)
        assert not gamma_huge_ratio.rule_r(a, cut / 1.001)


def test_aimed_points_stay_typable() -> None:
    rng = random.Random(4635)
    for _ in range(500):
        x, a, beta, cumulative = gamma_dist._huge_ratio_point(rng)
        assert isinstance(x, float) and 0 < x < 1e301  # Excel for the web refuses an x literal from 1E307
        assert isinstance(a, float) and not 1.1 < a < 1.11
        assert isinstance(beta, float) and 1e-100 <= beta <= 1e-8
        assert isinstance(cumulative, bool)
        assert x / beta <= sys.float_info.max


def test_fits_are_registered() -> None:
    assert {"huge-ratio-gamma", "huge-ratio-chisq", "huge-ratio-chisq-rt"} <= set(registry())
