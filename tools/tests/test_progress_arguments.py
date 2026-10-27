"""The progress page's `arguments` block (T-0077)."""

from __future__ import annotations

from typing import Any

import pytest

from rustsheet_tools.progress_report import arguments
from rustsheet_tools.progress_report.model import ProgressReportError, ReportShapeError


def report(*shares: float | None) -> dict[str, Any]:
    return {"argument_coverage": [{"name": f"F{i}", "single": s} for i, s in enumerate(shares)]}


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


def test_block_without_counted_functions() -> None:
    assert "No in-scope function" in arguments.block(arguments.load(report(None)))


def test_an_older_report_is_a_shape_error() -> None:
    with pytest.raises(ReportShapeError, match="T-0077"):
        arguments.load({})


@pytest.mark.parametrize("share", [1.5, -0.1, True, "0.5"])
def test_a_share_outside_zero_to_one_fails(share: object) -> None:
    with pytest.raises(ProgressReportError, match="not a share"):
        arguments.load({"argument_coverage": [{"name": "ABS", "single": share}]})
