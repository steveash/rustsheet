"""The progress page's `arguments` block (SPEC-0022 later phase, T-0077, T-0083).

Counts the in-scope functions per band of single-argument coverage: of the argument classes each
catalog parameter's mode is expected to see, the share some Excel-evidence cell passes it; beside
it, pairwise coverage: of the pairs of two parameters and an expected class of each, the share one
call passes together. The classes, the expected set per mode and each function's shares are the
eval report's (report.json `argument_coverage[]`, crates/rustsheet-eval/src/argument_coverage.rs);
this module only checks the shape, counts bands and takes means and medians.
"""

from __future__ import annotations

import statistics
from dataclasses import dataclass
from typing import Any

from rustsheet_tools.progress_report.markdown import pct, table
from rustsheet_tools.progress_report.model import EN, ProgressReportError, field, items

TASK = "T-0077"
PAIRWISE_TASK = "T-0083"
# Single-argument coverage bands: (label, low, high) in percent, low inclusive, high exclusive;
# the last band is exactly 100%.
BANDS: tuple[tuple[str, float, float], ...] = (
    ("< 50%", 0.0, 50.0),
    (f"50{EN}74%", 50.0, 75.0),
    (f"75{EN}89%", 75.0, 90.0),
    (f"90{EN}99%", 90.0, 100.0),
    ("100%", 100.0, 101.0),
)
LOWEST = 10
# The one-off assessment of 2026-10-25 (T-0077, T-0083 Context): its average single-argument
# coverage and its median pairwise coverage.
ASSESSMENT = "94%"
ASSESSMENT_PAIRWISE_MEDIAN = "44%"


@dataclass(frozen=True)
class FunctionCoverage:
    """One in-scope function's share of expected argument classes seen (0 to 1), and of expected
    class pairs seen together (`None` with fewer than two counted parameters)."""

    name: str
    share: float
    pairwise: float | None


@dataclass(frozen=True)
class Coverage:
    """The functions with a share, and how many have no counted parameter (PI, NOW)."""

    functions: list[FunctionCoverage]
    uncounted: int


def load(report: dict[str, Any]) -> Coverage:
    """report.json `argument_coverage[]`; a report without it predates T-0077 (stale page)."""
    rows = items(report, "argument_coverage", "report.json", TASK)
    functions = []
    uncounted = 0
    for i, row in enumerate(rows):
        where = f"report.json argument_coverage[{i}]"
        name = field(row, "name", where)
        share = _share(field(row, "single", where), f"{where}.single")
        pairwise = _share(field(row, "pairwise", where, PAIRWISE_TASK), f"{where}.pairwise")
        if share is None:
            uncounted += 1
            continue
        functions.append(FunctionCoverage(str(name), share, pairwise))
    return Coverage(functions, uncounted)


def _share(value: Any, where: str) -> float | None:
    """`value` checked to be `None` or a share between 0 and 1."""
    if value is None:
        return None
    if isinstance(value, bool) or not isinstance(value, int | float) or not 0 <= value <= 1:
        msg = f"{where} is not a share between 0 and 1: {value!r}"
        raise ProgressReportError(msg)
    return float(value)


def band_counts(c: Coverage) -> list[tuple[str, int]]:
    """Functions per coverage band (`BANDS`), in band order."""
    return [(label, sum(1 for f in c.functions if low <= 100 * f.share < high)) for label, low, high in BANDS]


def block(c: Coverage) -> str:
    """The block's Markdown."""
    n = len(c.functions)
    head = "## Argument coverage\n\nIndicative, rule-based expected types. "
    if n == 0:
        return head + "No in-scope function has a counted parameter in this run.\n"
    mean = statistics.fmean(f.share for f in c.functions)
    median = statistics.median(f.share for f in c.functions)
    rows = [[label, f"{k:,}", pct(k, n)] for label, k in band_counts(c)]
    rows.append(["**All**", f"**{n:,}**", ""])
    lowest = sorted(c.functions, key=lambda f: (f.share, f.name))[:LOWEST]
    low = ", ".join(f"{f.name} ({100 * f.share:.0f}%)" for f in lowest)
    return (
        head + "Each catalog parameter expects argument classes by its mode: a `range` parameter a "
        "reference; `lambda` and `name` parameters none (not counted); every other mode all eleven "
        "(positive number, zero, negative, fraction, numeric text, text, empty text, boolean, error, "
        "blank, range or array). A class is seen when an Excel-evidence cell passes it: a literal "
        "as written, a single-cell reference by the referenced cell's value; nested calls and "
        "operators are not classified (report.json `argument_coverage[]`).\n\n"
        f"Average single-argument coverage is **{100 * mean:.1f}%** (median {100 * median:.1f}%) "
        f"over {n:,} functions; {c.uncounted:,} have no counted parameter. The one-off assessment "
        f"of 2026-10-25 measured {ASSESSMENT} with its own classifier. Lowest: {low}.\n\n"
        + _pairwise(c)
        + table(["Coverage", "Functions", "Share"], rows)
        + "\n"
    )


def _pairwise(c: Coverage) -> str:
    """The pairwise paragraph: mean and median over the functions with two counted parameters."""
    shares = [f.pairwise for f in c.functions if f.pairwise is not None]
    if not shares:
        return "No in-scope function has two counted parameters for pairwise coverage.\n\n"
    return (
        "Pairwise coverage counts, for each two counted parameters, the pairs of an expected class "
        "of each that one Excel-evidence call passes together. "
        f"Average pairwise coverage is **{100 * statistics.fmean(shares):.1f}%** "
        f"(median {100 * statistics.median(shares):.1f}%) over {len(shares):,} functions with two "
        f"or more counted parameters; the one-off assessment of 2026-10-25 measured a "
        f"{ASSESSMENT_PAIRWISE_MEDIAN} median.\n\n"
    )
