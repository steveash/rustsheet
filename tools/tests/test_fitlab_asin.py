"""The ASIN fit (T-1797): the 2 - y formulation and the scores numbers.md quotes."""

from __future__ import annotations

import math

import pytest

from rustsheet_tools.fitlab import fit
from rustsheet_tools.fitlab.exact import X87
from rustsheet_tools.fitlab.kernels import asin, registry


def test_two_minus_y_is_not_odd() -> None:
    kernel = asin.two_minus_y(X87.sqrt)
    # probes/asin-argument-sweep B109, B91, B252, B256 (excel-graph).
    assert kernel(0.09) == 0.09012194501459526
    assert kernel(-0.09) == -0.09012194501459525
    assert kernel(0.999) == 1.526071239626163
    assert kernel(-0.999) == -1.5260712396261655
    assert kernel(1.0) == math.pi / 2


@pytest.mark.usefixtures("real_cases")
def test_scores_match_numbers_md() -> None:
    result = fit.run(registry()["asin"], engine=False)
    scores = {s.name: s.exact for s in result.scores}
    assert len(result.sweep.cells) == 570
    assert scores["atan(x/sqrt(y(2-y))), y=1-x, sqrt rounded twice"] == 570
    assert scores["atan(x/sqrt(y(2-y))), y=1-x"] == 567
    assert scores["atan(x/sqrt((1-x)(1+x))), x below 2^-28"] == 468
    assert scores["libm asin"] == 390
