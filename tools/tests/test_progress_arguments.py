"""The progress page's `arguments` block (T-0077)."""

from __future__ import annotations

from typing import Any

import pytest

from rustsheet_tools.progress_report import arguments
from rustsheet_tools.progress_report.model import ProgressReportError, ReportShapeError


def report(*shares: float | None, pairwise: tuple[float | None, ...] = ()) -> dict[str, Any]:
    pairs = pairwise or (None,) * len(shares)
    return {
        "argument_coverage": [
            {"name": f"F{i}", "single": s, "pairwise": p}
            for i, (s, p) in enumerate(zip(shares, pairs, strict=True))
        ]
    }


def test_functions_fall_in_coverage_bands() -> None:
    c = arguments.load(report(0.2, 0.5, 0.749, 0.9, 0.999, 1.0, None))
    assert c.uncounted == 1
    assert [k for _, k in arguments.band_counts(c)] == [1, 2, 0, 2, 1]


def test_block_shows_the_average_the_method_note_and_the_lowest() -> None:
    text = arguments.block(arguments.load(report(0.25, 0.75, None)))
    assert text.startswith("## Argument coverage\n\nIndicative, rule-based expected types.")
    assert "Average single-argument coverage is **50.0%** (median 50.0%) over 2 functions; 1 have" in text
    assert "Lowest: F0 (25%), F1 (75%)." in text
    assert "| **All** | **2** |  |" in text
    assert "No in-scope function has two counted parameters" in text


def test_block_shows_pairwise_mean_and_median_beside_the_single_ones() -> None:
    c = arguments.load(report(0.5, 0.9, 1.0, 0.8, pairwise=(0.1, 0.2, None, 0.6)))
    assert [f.pairwise for f in c.functions] == [0.1, 0.2, None, 0.6]
    text = arguments.block(c)
    assert "Average pairwise coverage is **30.0%** (median 20.0%) over 3 functions" in text
    assert "measured a 44% median" in text
    assert text.index("Average single-argument") < text.index("Average pairwise") < text.index("| Coverage |")


def test_block_without_counted_functions() -> None:
    assert "No in-scope function" in arguments.block(arguments.load(report(None)))


def test_an_older_report_is_a_shape_error() -> None:
    with pytest.raises(ReportShapeError, match="T-0077"):
        arguments.load({})


def test_a_report_without_pairwise_shares_predates_t0083() -> None:
    with pytest.raises(ReportShapeError, match="T-0083"):
        arguments.load({"argument_coverage": [{"name": "ABS", "single": 0.5}]})


@pytest.mark.parametrize("key", ["single", "pairwise"])
@pytest.mark.parametrize("share", [1.5, -0.1, True, "0.5"])
def test_a_share_outside_zero_to_one_fails(key: str, share: object) -> None:
    row = {"name": "ABS", "single": 0.5, "pairwise": 0.5} | {key: share}
    with pytest.raises(ProgressReportError, match=f"{key} is not a share"):
        arguments.load({"argument_coverage": [row]})
